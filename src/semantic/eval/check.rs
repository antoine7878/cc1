use crate::ast::visit::{Visitor, walk_expression, walk_translation_unit};
use crate::ast::{Expression, ExpressionNode};
use crate::context::Context;
use crate::semantic::{Sema, ice};

struct ConstChecker<'a> {
    sema: &'a mut Sema,
}

impl Visitor for ConstChecker<'_> {
    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        walk_expression(self, ctx, node);
        if matches!(node.id.resolve(ctx), Expression::ConstantExpression(_)) {
            ice::eval_constant(self.sema, ctx, node);
        }
    }
}

pub fn run(sema: &mut Sema, ctx: &Context) {
    let mut checker = ConstChecker { sema };
    walk_translation_unit(&mut checker, ctx, &ctx.ast);
}
