use crate::arena::OptionPoisoned;
use crate::ast::{ExpressionNode, Storage, Type, UnaryOp};
use crate::semantic::ExpressionKind::{LValue, RValue};
use crate::semantic::resolution::expression::*;
use crate::semantic::{
    Diagnosis, QualifiedType, ResolvedExpression, ResolvedType, Sema, SymbolId, SymbolResolver, cast, constrain,
    declaration,
};

pub fn inc_dec(sema: &mut Sema, e: &ExpressionNode, op: UnaryOp) -> R {
    with_converted(sema, [e], |sema, [re]| {
        constrain::expression::check_assignable(sema, re.kind, re.ty).into_result()?;
        let non_object_pointee = match re.ty.id.resolve_in(sema) {
            ResolvedType::Pointer(inner) if !inner.is_object(sema) => Some((*inner, inner.is_function(sema))),
            _ => None,
        };
        constrain::expression::check_inc_dec(op, re.ty.is_scalar(sema), non_object_pointee, re.ty).into_result()?;
        Ok((re.casted_ty(), RValue))
    })
}

pub fn sign(sema: &mut Sema, e: &ExpressionNode) -> R {
    with_converted(sema, [e], |sema, [re]| {
        if !re.casted_ty().is_arithmetic(sema) {
            return Err(Diagnosis::InvalidUnary(re.ty));
        }
        cast::promote(sema, re);
        Ok((re.casted_ty(), RValue))
    })
}

pub fn address(sema: &mut Sema, e: &ExpressionNode) -> R {
    let id = sema.expr_bindings.get(e.id).copied();
    with_ops(sema, [e], |sema, [re]| address_type(sema, re, id))
}

fn address_type(sema: &mut Sema, re: &mut ResolvedExpression, sym: Option<SymbolId>) -> R {
    let is_register = sym.is_some_and(|id| id.resolve_in(sema).storage == Some(Storage::Register));
    constrain::expression::check_address_of(
        re.kind,
        re.casted_ty().is_function(sema),
        is_register,
        is_bit_field(sema, sym),
        re.ty,
    )
    .into_result()?;
    let ty = sema.types.pointer(re.ty);
    let qty = QualifiedType::plain(ty);
    Ok((qty, RValue))
}

pub fn indirection(sema: &mut Sema, e: &ExpressionNode) -> R {
    with_converted(sema, [e], |sema, [re]| {
        let ResolvedType::Pointer(inner) = re.casted_ty().id.resolve_in(sema) else {
            return Err(Diagnosis::IndirectionNotPointer(re.ty));
        };
        if inner.is_void(sema) {
            return Err(Diagnosis::IndirectionToVoid);
        }
        let kind = if inner.is_function(sema) { RValue } else { LValue };
        Ok((*inner, kind))
    })
}

pub fn bit_not(sema: &mut Sema, e: &ExpressionNode) -> R {
    with_converted(sema, [e], |sema, [re]| {
        if !re.casted_ty().is_integral(sema) {
            return Err(Diagnosis::InvalidUnary(re.ty));
        }
        cast::promote(sema, re);
        Ok((re.casted_ty(), RValue))
    })
}

pub fn logic_not(sema: &mut Sema, e: &ExpressionNode) -> R {
    with_converted(sema, [e], |sema, [re]| {
        if !re.casted_ty().is_scalar(sema) {
            return Err(Diagnosis::InvalidUnary(re.ty));
        }
        int_rvalue(sema)
    })
}

pub fn cast(resolver: &mut SymbolResolver, node: &ExpressionNode, ty_node: &Type, operand: &ExpressionNode) -> R {
    let base = declaration::base_type(resolver, &ty_node.specifiers, &node.span);
    let (qualif, _) = declaration::declared_type(resolver, base, &ty_node.declarator).ok_poisoned()?;
    let is_null = is_null_pointer_constant(resolver.sema, operand);
    with_converted(resolver.sema, [operand], |sema, [re]| {
        let ty = qualif.id.resolve_in(sema);
        if !ty.is_void() {
            let from = re.casted_ty().id.resolve_in(sema);
            if !ty.is_scalar(sema) {
                return Err(Diagnosis::CastToNonScalar);
            }
            if !from.is_scalar(sema) {
                return Err(Diagnosis::CastOfNonScalar);
            }
            if ty.is_pointer() != from.is_pointer() && (ty.is_floating() || from.is_floating()) {
                return Err(Diagnosis::InvalidOperand);
            }
            cast::convert(sema, re, qualif.id, is_null);
        }
        Ok((qualif, RValue))
    })
}
