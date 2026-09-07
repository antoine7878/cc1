use crate::arena::ResolveWith;
use crate::ast::{BinaryOp, ExpressionNode, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::{
    AssignmentContext, Diagnosis, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, Sema, cast, ice,
};

use super::arithmetic::{multiplicative_types, shift_types};
use super::compare::bitwise_type;
use super::operand::{R, check_assignable, is_null_pointer_constant, operands};

pub fn init(
    sema: &mut Sema,
    ctx: &Context,
    l_ty: QualifiedType,
    init_node: &ExpressionNode,
    assign_ctx: AssignmentContext,
) -> R {
    let is_null = is_null_pointer_constant(sema, ctx, init_node);
    let mut ops = operands(sema, [init_node])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &init_node.span);
    let mut l_re = ResolvedExpression::new(l_ty, ExpressionKind::LValue);
    let out = cast::assignment_conversion(sema, &mut l_re, re, is_null, assign_ctx);
    if matches!(assign_ctx, AssignmentContext::Return)
        && let Some(cast) = re.casts.last_mut()
    {
        cast.to.is_volatile = false;
        cast.to.is_const = false;
    }
    out.map(|q| (q, RValue))
}

pub(super) fn simple_assignment(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let is_null = is_null_pointer_constant(sema, ctx, e2);
    let mut ops = operands(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, rhs, &e2.span);
    simple_assignation_type(sema, lhs, rhs, is_null)
}

fn simple_assignation_type(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    is_null: bool,
) -> R {
    check_assignable(lhs)?;
    let q = cast::assignment_conversion(sema, lhs, rhs, is_null, AssignmentContext::Assignment)?;
    Ok((q, ExpressionKind::RValue))
}

pub(super) fn additive_assignment(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let mut ops = operands(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, rhs, &e2.span);
    additive_assignation_type(sema, lhs, rhs, &e1.span)
}

fn additive_assignation_type(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    span: &Span,
) -> R {
    check_assignable(lhs)?;
    let target = lhs.ty.unqualified();
    let l = lhs.ty.id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);

    if matches!(l, ResolvedType::Pointer(inner) if inner.is_object(sema)) && r.is_integral(sema) {
        cast::lvalue_conversion(sema, lhs, span);
        cast::pointer_integer_arithmetic(sema, lhs, rhs)?;
    } else if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::lvalue_conversion(sema, lhs, span);
        cast::usual_arithmetic(sema, lhs, rhs)?;
        lhs.result_cast = cast::arithmetic_conversion(sema, lhs.casted_ty(), target);
    } else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.casted_ty()));
    }
    Ok((target, ExpressionKind::RValue))
}

pub(super) fn coumpound_assignment(
    sema: &mut Sema,
    ctx: &Context,
    op: &BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
) -> R {
    let count = ice::try_fold(sema, ctx, e2);
    let mut ops = operands(sema, [e1, e2])?;
    let (sema, [lhs, rhs]) = ops.parts();
    cast::lvalue_conversion(sema, rhs, &e2.span);
    coumpound_assignation_type(sema, op, lhs, rhs, &e1.span, count)
}

fn coumpound_assignation_type(
    sema: &mut Sema,
    op: &BinaryOp,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    span: &Span,
    count: Option<Value>,
) -> R {
    check_assignable(lhs)?;
    let target = lhs.ty.unqualified();
    cast::lvalue_conversion(sema, lhs, span);
    match op {
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => multiplicative_types(sema, op, lhs, rhs),
        BinaryOp::Left | BinaryOp::Right => shift_types(sema, lhs, rhs, count),
        BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => bitwise_type(sema, lhs, rhs),
        _ => unreachable!(),
    }?;
    lhs.result_cast = cast::arithmetic_conversion(sema, lhs.casted_ty(), target);
    Ok((target, ExpressionKind::RValue))
}
