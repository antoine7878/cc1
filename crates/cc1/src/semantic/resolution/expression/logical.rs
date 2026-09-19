use crate::ast::ExpressionNode;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diagnostic, Sema};

pub fn logical(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        if !lhs.casted_ty().is_scalar(sema) || !rhs.casted_ty().is_scalar(sema) {
            return Err(Diagnostic::InvalidBinaryOperand(lhs.ty, rhs.ty));
        }
        int_rvalue(sema)
    })
}
