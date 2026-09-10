use crate::ast::visit::{Visitor, walk_expression, walk_translation_unit};
use crate::ast::{Expression, ExpressionNode};
use crate::context::{Context, ctx};
use crate::semantic::{Sema, ice};

struct ConstChecker<'a> {
    sema: &'a mut Sema,
}

impl Visitor for ConstChecker<'_> {
    fn visit_expression(&mut self, node: &ExpressionNode) {
        let ctx = ctx();
        walk_expression(self, node);
        if matches!(node.id.resolve(), Expression::ConstantExpression(_)) {
            ice::eval_constant(self.sema, ctx, node);
        }
    }
}

pub fn check_constants(sema: &mut Sema, ctx: &Context) {
    sema.size_tables(&ctx.arenas);
    let mut checker = ConstChecker { sema };
    walk_translation_unit(&mut checker, &ctx.ast);
}
