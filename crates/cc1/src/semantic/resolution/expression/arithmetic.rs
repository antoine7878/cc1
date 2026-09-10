use crate::ast::{BinaryOp, ExpressionNode, Value};
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diag, DiagCollector, Diagnosis, ResolvedExpression, ResolvedType, Sema, cast, ice};
use libft::Span;

pub fn multiplicative(sema: &mut Sema, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        multiplicative_types(sema, op, lhs, rhs)
    })
}

pub fn multiplicative_types(
    sema: &mut Sema,
    op: &BinaryOp,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
) -> R {
    let l = lhs.casted_ty().id.resolve_with(sema);
    let r = rhs.casted_ty().id.resolve_with(sema);
    if !match op {
        BinaryOp::Mul | BinaryOp::Div => l.is_arithmetic(sema) && r.is_arithmetic(sema),
        BinaryOp::Mod => l.is_integral(sema) && r.is_integral(sema),
        _ => unreachable!(),
    } {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.casted_ty(), rhs.casted_ty()));
    }
    cast::usual_arithmetic(sema, lhs, rhs)
}

pub fn additive(sema: &mut Sema, op: &BinaryOp, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        match (
            op,
            lhs.casted_ty().id.resolve_with(sema),
            rhs.casted_ty().id.resolve_with(sema),
        ) {
            (_, l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => cast::usual_arithmetic(sema, lhs, rhs),
            (_, ResolvedType::Pointer(_), o) if o.is_integral(sema) => cast::pointer_integer_arithmetic(sema, lhs, rhs),
            (BinaryOp::Add, o, ResolvedType::Pointer(_)) if o.is_integral(sema) => {
                cast::pointer_integer_arithmetic(sema, rhs, lhs)
            }
            (BinaryOp::Sub, ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => {
                cast::pointer_minus_pointer(sema, lhs, rhs)
            }
            _ => Err(Diagnosis::InvalidOperand),
        }
    })
}

pub fn shift(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let count = ice::try_fold(sema, e2);
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        shift_types(sema, lhs, rhs, count, &e2.span)
    })
}

pub fn shift_types(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    count: Option<Value>,
    span: &Span,
) -> R {
    let l = lhs.casted_ty().id.resolve_with(sema);
    let r = rhs.casted_ty().id.resolve_with(sema);
    if !l.is_integral(sema) || !r.is_integral(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.casted_ty(), rhs.casted_ty()));
    }
    cast::promote(sema, lhs);
    cast::promote(sema, rhs);
    let l = lhs.casted_ty().id.resolve_with(sema);
    let l_layout = sema.target.layout(l).unwrap();
    let Some(count) = count else { return Ok((lhs.casted_ty(), RValue)) };
    if count.is_negative() {
        sema.add_diag(Diag::err((), Diagnosis::ShiftCountNegative), span);
    } else if count.is_greater_or_eq(l_layout.size * sema.target.byte_size) {
        sema.add_diag(Diag::err((), Diagnosis::ShiftCountOutOfRange), span);
    }
    Ok((lhs.casted_ty(), RValue))
}
