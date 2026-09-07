use crate::arena::ResolveWith;
use crate::ast::ExpressionNode;
use crate::context::Context;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::{Diagnosis, QualifiedType, ResolvedExpression, ResolvedType, Sema, cast};

use super::operand::{R, is_null_pointer_constant, with_converted};

pub(super) fn relational(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| relational_type(sema, lhs, rhs))
}

fn relational_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
    let l = lhs.casted_ty().id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    let ret = Ok((ty, RValue));
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::usual_arithmetic(sema, lhs, rhs)?;
        return ret;
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    };
    if !i1.is_compatible_ignoring_qualifiers(sema, i2) {
        return Err(Diagnosis::InvalidComparison(lhs.ty, rhs.ty));
    }
    if i1.is_object(sema) && i2.is_object(sema) {
        return ret;
    }
    if !i1.is_complete(sema) && !i2.is_complete(sema) {
        return ret;
    }
    if i1.is_function(sema) && i2.is_function(sema) {
        return Err(Diagnosis::OrderedFunctionPointers(lhs.ty, rhs.ty));
    }
    Err(Diagnosis::MixedCompletenessComparison(lhs.ty, rhs.ty))
}

pub(super) fn equality(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    let null1 = is_null_pointer_constant(sema, ctx, e1);
    let null2 = is_null_pointer_constant(sema, ctx, e2);
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| {
        equality_type(sema, lhs, rhs, null1, null2)
    })
}

fn equality_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression, n1: bool, n2: bool) -> R {
    let l_ty = lhs.casted_ty();
    let r_ty = rhs.casted_ty();
    let l = l_ty.id.resolve(sema);
    let r = r_ty.id.resolve(sema);
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    let ret = Ok((ty, RValue));
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        cast::usual_arithmetic(sema, lhs, rhs)?;
        return ret;
    }
    if l.is_pointer() && n2 {
        cast::convert(sema, rhs, l_ty.id, true);
        return ret;
    }
    if r.is_pointer() && n1 {
        cast::convert(sema, lhs, r_ty.id, true);
        return ret;
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    };
    if i1.is_compatible_ignoring_qualifiers(sema, i2) {
        return ret;
    }
    if i1.is_void(sema) && !i2.is_function(sema) {
        cast::convert(sema, rhs, l_ty.id, false);
        return ret;
    }
    if i2.is_void(sema) && !i1.is_function(sema) {
        cast::convert(sema, lhs, r_ty.id, false);
        return ret;
    }
    Err(Diagnosis::InvalidComparison(lhs.ty, rhs.ty))
}

pub(super) fn bitwise(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| bitwise_type(sema, lhs, rhs))
}

pub(super) fn bitwise_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
    if !lhs.casted_ty().is_integral(sema) || !rhs.casted_ty().is_integral(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }
    cast::usual_arithmetic(sema, lhs, rhs)
}

pub(super) fn logic(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode) -> R {
    with_converted(sema, [e1, e2], |sema, [lhs, rhs]| logic_type(sema, lhs, rhs))
}

fn logic_type(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) -> R {
    if !lhs.casted_ty().is_scalar(sema) || !rhs.casted_ty().is_scalar(sema) {
        return Err(Diagnosis::InvalidBinaryOperand(lhs.ty, rhs.ty));
    }
    let ty = QualifiedType::new(sema.builtins.int, false, false);
    Ok((ty, RValue))
}
