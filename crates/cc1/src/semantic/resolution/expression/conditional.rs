use crate::arena::{OptionPoisoned, ResolveWith};
use crate::ast::ExpressionNode;
use crate::context::Context;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::resolution::expression::*;
use crate::semantic::{Diagnosis, QualifiedType, Sema, cast};

pub fn conditional(sema: &mut Sema, ctx: &Context, e1: &ExpressionNode, e2: &ExpressionNode, e3: &ExpressionNode) -> R {
    let null2 = is_null_pointer_constant(sema, ctx, e2);
    let null3 = is_null_pointer_constant(sema, ctx, e3);
    with_converted(sema, [e1, e2, e3], |sema, [condition, lhs, rhs]| {
        if !condition.casted_ty().is_scalar(sema) {
            return Err(Diagnosis::NotScalar(condition.ty));
        }
        let l_ty = lhs.casted_ty();
        let r_ty = rhs.casted_ty();
        let l = l_ty.id.resolve_in(sema);
        let r = r_ty.id.resolve_in(sema);
        if l.is_arithmetic(sema) && r.is_arithmetic(sema) {
            return cast::usual_arithmetic(sema, lhs, rhs);
        }
        if l.is_tag() && r.is_tag() && l_ty.is_compatible(sema, &r_ty) {
            return Ok((l_ty, RValue));
        }
        if l.is_void() && r.is_void() {
            return Ok((QualifiedType::plain(sema.builtins.void), RValue));
        }
        let were_pointers = both_pointers(sema, lhs, rhs);
        match reconcile_pointers(sema, lhs, rhs, null2, null3) {
            Some(PointerMatch::Converted) => Ok((lhs.casted_ty(), RValue)),
            Some(PointerMatch::Compatible(i1, i2)) => {
                let inner = i1.unqualified().composite(sema, &i2.unqualified()).ok_poisoned()?;
                let inner = QualifiedType::new(inner.id, i1.is_const || i2.is_const, i1.is_volatile || i2.is_volatile);
                let ty = sema.types.pointer(inner);
                Ok((QualifiedType::plain(ty), RValue))
            }
            None if were_pointers => Err(Diagnosis::PointerMismatch(lhs.ty, rhs.ty)),
            None => Err(Diagnosis::IncompatibleOperands(lhs.ty, rhs.ty)),
        }
    })
}
