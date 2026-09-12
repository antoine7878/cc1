use crate::ast::visit::{Visitor, walk_expression, walk_translation_unit};
use crate::ast::{Expression, ExpressionNode};
use crate::context::ctx;
use crate::semantic::{Sema, ice};

struct ConstChecker<'a> {
    sema: &'a mut Sema,
}

impl Visitor for ConstChecker<'_> {
    fn visit_expression(&mut self, node: &ExpressionNode) {
        if matches!(node.id.resolve(), Expression::ConstantExpression(_)) {
            ice::eval_constant(self.sema, node);
            return;
        }
        walk_expression(self, node);
        if !self.sema.expr_consts.seen(node.id) {
            let value = ice::try_fold(self.sema, node);
            self.sema.expr_consts.set(node.id, value);
        }
    }
}

pub fn check_constants(sema: &mut Sema) {
    let ctx = ctx();
    sema.size_tables(&ctx.arenas);
    let mut checker = ConstChecker { sema };
    walk_translation_unit(&mut checker, &ctx.ast);
}
