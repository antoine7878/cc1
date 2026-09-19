use crate::ast::ExpressionNode;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diagnostic, ResolvedExpression, Sema, cast};

pub fn bitwise(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| bitwise_types(sema, lhs, rhs))
}

pub fn bitwise_types(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> ExprResult {
    if !lhs.casted_ty().is_integral(sema) || !rhs.casted_ty().is_integral(sema) {
        return Err(Diagnostic::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }
    cast::usual_arithmetic(sema, lhs, rhs)
}
