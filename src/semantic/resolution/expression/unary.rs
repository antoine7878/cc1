use crate::arena::{OptionPoisoned, ResolveWith};
use crate::ast::{ExpressionNode, Storage, Type, UnaryOp};
use crate::context::Context;
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::{Diagnosis, QualifiedType, ResolvedExpression, ResolvedType, Sema, SymbolId, cast, declaration};

use super::operand::{R, check_assignable, is_bit_field, is_null_pointer_constant, operands};

pub(super) fn inc_dec(sema: &mut Sema, e: &ExpressionNode, op: UnaryOp) -> R {
    let mut ops = operands(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    check_assignable(re)?;
    if let ResolvedType::Pointer(inner) = re.ty.id.resolve(sema)
        && !inner.is_object(sema)
    {
        return Err(match inner.is_function(sema) {
            true => Diagnosis::BadPostIncDec(op, re.ty),
            false => Diagnosis::IncompleteType(*inner),
        });
    }
    if !re.ty.is_scalar(sema) {
        return Err(Diagnosis::BadPostIncDec(op, re.ty));
    }
    Ok((re.casted_ty(), RValue))
}

pub(super) fn sign(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = operands(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    if !re.casted_ty().is_arithmetic(sema) {
        return Err(Diagnosis::InvalidUnary(re.ty));
    }
    cast::promote(sema, re);
    Ok((re.casted_ty(), RValue))
}

pub(super) fn address(sema: &mut Sema, e: &ExpressionNode) -> R {
    let id = sema.expr_bindings.get(e.id).copied();
    let mut ops = operands(sema, [e])?;
    let (sema, [re]) = ops.parts();
    address_type(sema, re, id)
}

fn address_type(sema: &mut Sema, re: &mut ResolvedExpression, sym: Option<SymbolId>) -> R {
    if re.kind == RValue && !re.casted_ty().is_function(sema) {
        return Err(Diagnosis::RValueAddress(re.ty));
    }
    if let Some(id) = sym {
        let sym = id.resolve(sema);
        if sym.storage == Some(Storage::Register) {
            return Err(Diagnosis::RegisterAddress);
        }
    }
    if is_bit_field(sema, sym) {
        return Err(Diagnosis::BitFieldAddress);
    }
    let ty = sema.types.pointer(re.ty);
    let qty = QualifiedType::new(ty, false, false);
    Ok((qty, RValue))
}

pub(super) fn indirection(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = operands(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    let ResolvedType::Pointer(inner) = re.casted_ty().id.resolve(sema) else {
        return Err(Diagnosis::IndirectionNotPointer(re.ty));
    };
    if inner.is_void(sema) {
        return Err(Diagnosis::IncompleteType(*inner));
    }
    let kind = if inner.is_function(sema) { RValue } else { LValue };
    Ok((*inner, kind))
}

pub(super) fn bit_not(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = operands(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    if !re.casted_ty().is_integral(sema) {
        return Err(Diagnosis::InvalidUnary(re.ty));
    }
    cast::promote(sema, re);
    Ok((re.casted_ty(), RValue))
}

pub(super) fn logic_not(sema: &mut Sema, e: &ExpressionNode) -> R {
    let mut ops = operands(sema, [e])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &e.span);
    if !re.casted_ty().is_scalar(sema) {
        return Err(Diagnosis::InvalidUnary(re.ty));
    }
    let qty = QualifiedType::new(sema.builtins.int, false, false);
    Ok((qty, RValue))
}

pub(super) fn cast(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
    ty_node: &Type,
    operand: &ExpressionNode,
) -> R {
    let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
    let (qualif, _) = declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_poisoned()?;
    let is_null = is_null_pointer_constant(sema, ctx, operand);
    let mut ops = operands(sema, [operand])?;
    let (sema, [re]) = ops.parts();
    cast::lvalue_conversion(sema, re, &operand.span);
    cast_type(sema, qualif, re, is_null)
}

fn cast_type(sema: &mut Sema, qualif: QualifiedType, re: &mut ResolvedExpression, is_null: bool) -> R {
    let ty = qualif.id.resolve(sema);
    if !ty.is_void() {
        let from = re.casted_ty().id.resolve(sema);
        if !ty.is_scalar(sema) || !from.is_scalar(sema) {
            return Err(Diagnosis::CastToNonScalar);
        }
        if ty.is_pointer() != from.is_pointer() && (ty.is_floating() || from.is_floating()) {
            return Err(Diagnosis::InvalidOperand);
        }
        cast::convert(sema, re, qualif.id, is_null);
    }
    Ok((qualif, RValue))
}
