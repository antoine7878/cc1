use crate::arena::ResolveMutWith;
use crate::ast::visit::{Visitor, walk_expression, walk_translation_unit};
use crate::ast::{Expression, ExpressionNode};
use crate::context::Context;
use crate::semantic::Sema;

struct UseMarker<'a> {
    sema: &'a mut Sema,
}

impl Visitor for UseMarker<'_> {
    fn visit_expression(&mut self, node: &ExpressionNode) {
        if matches!(
            node.id.resolve(),
            Expression::SizeofExpr(_) | Expression::SizeofType(_)
        ) {
            return;
        }
        walk_expression(self, node);
        if matches!(node.id.resolve(), Expression::Identifier(_))
            && let Some(sym_id) = self.sema.expr_bindings.get(node.id).copied()
        {
            sym_id.resolve_mut(self.sema).used = true;
        }
    }
}

pub fn mark_uses(sema: &mut Sema, ctx: &Context) {
    let mut marker = UseMarker { sema };
    walk_translation_unit(&mut marker, &ctx.ast);
}
