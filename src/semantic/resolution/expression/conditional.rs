use crate::arena::{OptionPoisoned, ResolveWith};
use crate::ast::ExpressionNode;
use crate::context::Context;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::{Diagnosis, QualifiedType, ResolvedExpression, ResolvedType, Sema, cast};

use super::operand::{R, is_null_pointer_constant, with_converted};

pub(super) fn conditional(
    sema: &mut Sema,
    ctx: &Context,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    e3: &ExpressionNode,
) -> R {
    let null2 = is_null_pointer_constant(sema, ctx, e2);
    let null3 = is_null_pointer_constant(sema, ctx, e3);
    with_converted(sema, [e1, e2, e3], |sema, [condition, lhs, rhs]| {
        conditional_type(sema, condition, lhs, rhs, null2, null3)
    })
}

fn conditional_type(
    sema: &mut Sema,
    conditional: &mut ResolvedExpression,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    n2: bool,
    n3: bool,
) -> R {
    if !conditional.casted_ty().is_scalar(sema) {
        return Err(Diagnosis::NotScalar(conditional.ty));
    }
    let l_ty = lhs.casted_ty();
    let r_ty = rhs.casted_ty();
    let l = l_ty.id.resolve(sema);
    let r = r_ty.id.resolve(sema);
    if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
        return cast::usual_arithmetic(sema, lhs, rhs);
    }
    if l.is_tag() && r.is_tag() && l_ty.is_compatible(sema, &r_ty) {
        return Ok((l_ty, RValue));
    }
    if l.is_void() && r.is_void() {
        let ty = QualifiedType::new(sema.builtins.void, false, false);
        return Ok((ty, RValue));
    }

    if l.is_pointer() && n3 {
        cast::convert(sema, rhs, l_ty.id, true);
        return Ok((lhs.casted_ty(), RValue));
    }
    if r.is_pointer() && n2 {
        cast::convert(sema, lhs, r_ty.id, true);
        return Ok((lhs.casted_ty(), RValue));
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return Err(Diagnosis::IncompatibleOperands(lhs.ty, rhs.ty));
    };

    if i1.is_compatible_ignoring_qualifiers(sema, i2) {
        let (i1, i2) = (*i1, *i2);
        let inner = i1.unqualified().composite(sema, &i2.unqualified()).ok_poisoned()?;
        let inner = QualifiedType::new(inner.id, i1.is_const || i2.is_const, i1.is_volatile || i2.is_volatile);
        let ty = sema.types.pointer(inner);
        return Ok((QualifiedType::new(ty, false, false), RValue));
    }

    if i1.is_void(sema) && !i2.is_function(sema) {
        cast::convert(sema, rhs, l_ty.id, false);
        return Ok((lhs.casted_ty(), RValue));
    }
    if i2.is_void(sema) && !i1.is_function(sema) {
        cast::convert(sema, lhs, r_ty.id, false);
        return Ok((lhs.casted_ty(), RValue));
    }
    Err(Diagnosis::PointerMismatch(lhs.ty, rhs.ty))
}
