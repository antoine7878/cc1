use crate::arena::ResolveMutWith;
use crate::ast::visit::{Visitor, walk_expression, walk_translation_unit};
use crate::ast::{Expression, ExpressionNode};
use crate::context::Context;
use crate::semantic::Sema;

struct UseMarker<'a> {
    sema: &'a mut Sema,
}

impl Visitor for UseMarker<'_> {
    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if matches!(
            node.id.resolve(ctx),
            Expression::SizeofExpr(_) | Expression::SizeofType(_)
        ) {
            return;
        }
        walk_expression(self, ctx, node);
        if matches!(node.id.resolve(ctx), Expression::Identifier(_))
            && let Some(sym_id) = self.sema.binding(node.id)
        {
            sym_id.resolve_mut(self.sema).used = true;
        }
    }
}

pub fn mark_uses(sema: &mut Sema, ctx: &Context) {
    let mut marker = UseMarker { sema };
    walk_translation_unit(&mut marker, ctx, &ctx.ast);
}
