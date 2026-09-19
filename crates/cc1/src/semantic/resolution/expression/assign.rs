use crate::ast::{BinaryOp, ExpressionNode};
use crate::semantic::ValueCategory::RValue;
use crate::semantic::resolution::expression::*;
use crate::semantic::{
    AssignmentContext, Diagnostic, QualifiedType, ResolvedExpression, ResolvedType, Sema, ValueCategory, cast,
    constraints, fold,
};

fn with_assign_ops<F>(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode, f: F) -> ExprResult
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression, &mut ResolvedExpression) -> ExprResult,
{
    with_ops(sema, [e1, e2], |sema, [lhs, rhs]| {
        cast::convert_operand(sema, rhs, &e2.span);
        f(sema, lhs, rhs)
    })
}

pub fn init(
    sema: &mut Sema,
    l_ty: QualifiedType,
    init_node: &ExpressionNode,
    assign_ctx: AssignmentContext,
) -> ExprResult {
    let is_null = is_null_pointer_constant(sema, init_node);
    with_converted(sema, [init_node], |sema, [re]| {
        let mut l_re = ResolvedExpression::new(l_ty, ValueCategory::LValue);
        let out = cast::assignment_conversion(sema, &mut l_re, re, is_null, assign_ctx);
        if matches!(assign_ctx, AssignmentContext::Return)
            && let Some(cast) = re.casts.last_mut()
        {
            cast.to.is_volatile = false;
            cast.to.is_const = false;
        }
        out.map(|q| (q, RValue))
    })
}

pub fn simple_assignment(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    let is_null = is_null_pointer_constant(sema, e2);
    with_assign_ops(sema, e1, e2, |sema, lhs, rhs| {
        constraints::expression::check_assignable(sema, lhs.kind, lhs.ty).into_result()?;
        let q = cast::assignment_conversion(sema, lhs, rhs, is_null, AssignmentContext::Assignment)?;
        Ok((q, RValue))
    })
}

pub fn additive_assignment(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    with_assign_ops(sema, e1, e2, |sema, lhs, rhs| {
        constraints::expression::check_assignable(sema, lhs.kind, lhs.ty).into_result()?;
        let target = lhs.ty.unqualified();
        let l = lhs.ty.id.resolve_with(sema);
        let r = rhs.casted_ty().id.resolve_with(sema);

        if matches!(l, ResolvedType::Pointer(inner) if inner.is_object(sema)) && r.is_integral(sema) {
            cast::convert_operand(sema, lhs, &e1.span);
            cast::pointer_integer_arithmetic(sema, lhs, rhs)?;
        } else if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
            cast::convert_operand(sema, lhs, &e1.span);
            cast::usual_arithmetic(sema, lhs, rhs)?;
            lhs.result_cast = cast::arithmetic_conversion(sema, lhs.casted_ty(), target);
        } else {
            return Err(Diagnostic::InvalidBinaryOperand(lhs.ty, rhs.casted_ty()));
        }
        Ok((target, RValue))
    })
}

pub fn compound_assignment(sema: &mut Sema, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> ExprResult {
    let count = fold::try_fold(sema, e2);
    with_assign_ops(sema, e1, e2, |sema, lhs, rhs| {
        constraints::expression::check_assignable(sema, lhs.kind, lhs.ty).into_result()?;
        let target = lhs.ty.unqualified();
        cast::convert_operand(sema, lhs, &e1.span);
        match op {
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative_types(sema, op, lhs, rhs),
            BinaryOp::Left | BinaryOp::Right => shift_types(sema, lhs, rhs, count, &e2.span),
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => bitwise_types(sema, lhs, rhs),
            _ => unreachable!(),
        }?;
        lhs.result_cast = cast::arithmetic_conversion(sema, lhs.casted_ty(), target);
        Ok((target, RValue))
    })
}
