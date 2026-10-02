use libft::Span;

use crate::arena::OptionPoisoned;
use crate::ast::{Expression, ExpressionNode, MemberOp, Storage, TypeName, UnaryOp};
use crate::semantic::ValueCategory::{LValue, RValue};
use crate::semantic::resolution::expression::*;
use crate::semantic::{
    Diag, Diagnostic, DiagnosticSink, QualifiedType, ResolvedExpression, ResolvedType, Resolver, Sema, SymbolId, cast,
    constraints, declaration,
};

pub fn inc_dec(sema: &mut Sema, e: &ExpressionNode, op: UnaryOp) -> ExprResult {
    with_converted(sema, [e], |sema, [re]| {
        constraints::expression::check_assignable(sema, re.kind, re.ty).into_result()?;
        let non_object_pointee = match re.ty.id.resolve_with(sema) {
            ResolvedType::Pointer(inner) if !inner.is_object(sema) => Some((*inner, inner.is_function(sema))),
            _ => None,
        };
        constraints::expression::check_inc_dec(op, re.ty.is_scalar(sema), non_object_pointee, re.ty).into_result()?;
        let target = re.ty.unqualified();
        cast::promote(sema, re);
        re.result_cast = cast::arithmetic_conversion(sema, re.casted_ty(), target);
        Ok((target, RValue))
    })
}

pub fn unary_sign(sema: &mut Sema, e: &ExpressionNode) -> ExprResult {
    with_converted(sema, [e], |sema, [re]| {
        if !re.casted_ty().is_arithmetic(sema) {
            return Err(Diagnostic::InvalidUnary(re.ty));
        }
        cast::promote(sema, re);
        Ok((re.casted_ty(), RValue))
    })
}

pub fn address(sema: &mut Sema, e: &ExpressionNode) -> ExprResult {
    let id = sema.expr_bindings.get(e.unparenthesized().id).copied();
    let object = object_symbol(sema, e);
    with_ops(sema, [e], |sema, [re]| address_type(sema, re, id, object))
}

fn object_symbol(sema: &Sema, e: &ExpressionNode) -> Option<SymbolId> {
    match e.id.resolve() {
        Expression::Member(MemberOp::Dot, base, _) | Expression::Block(base) => object_symbol(sema, base),
        _ => sema.expr_bindings.get(e.id).copied(),
    }
}

pub fn is_register_object(sema: &Sema, e: &ExpressionNode) -> bool {
    is_register_symbol(sema, object_symbol(sema, e))
}

fn is_register_symbol(sema: &Sema, object: Option<SymbolId>) -> bool {
    object.is_some_and(|id| id.resolve_with(sema).storage == Some(Storage::Register))
}

fn address_type(
    sema: &mut Sema,
    re: &mut ResolvedExpression,
    sym: Option<SymbolId>,
    object: Option<SymbolId>,
) -> ExprResult {
    let is_register = is_register_symbol(sema, object);
    constraints::expression::check_address_of(
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

pub fn indirection(sema: &mut Sema, e: &ExpressionNode, span: &Span) -> ExprResult {
    with_converted(sema, [e], |sema, [re]| {
        let &ResolvedType::Pointer(inner) = re.casted_ty().id.resolve_with(sema) else {
            return Err(Diagnostic::IndirectionNotPointer(re.ty));
        };
        if inner.is_void(sema) {
            sema.add_diag(Diag::err((), Diagnostic::IndirectionToVoid), span);
            return Ok((inner, RValue));
        }
        let kind = if inner.is_function(sema) { RValue } else { LValue };
        Ok((inner, kind))
    })
}

pub fn bit_not(sema: &mut Sema, e: &ExpressionNode) -> ExprResult {
    with_converted(sema, [e], |sema, [re]| {
        if !re.casted_ty().is_integral(sema) {
            return Err(Diagnostic::InvalidUnary(re.ty));
        }
        cast::promote(sema, re);
        Ok((re.casted_ty(), RValue))
    })
}

pub fn logical_not(sema: &mut Sema, e: &ExpressionNode) -> ExprResult {
    with_converted(sema, [e], |sema, [re]| {
        if !re.casted_ty().is_scalar(sema) {
            return Err(Diagnostic::InvalidUnary(re.ty));
        }
        int_rvalue(sema)
    })
}

pub fn cast(
    resolver: &mut Resolver,
    node: &ExpressionNode,
    ty_node: &TypeName,
    operand: &ExpressionNode,
) -> ExprResult {
    let base = declaration::base_type(resolver, &ty_node.specifiers, &node.span);
    let (qualif, _) = declaration::declared_type(resolver, base, &ty_node.declarator).ok_poisoned()?;
    let is_null = is_null_pointer_constant(resolver.sema, operand);
    with_converted(resolver.sema, [operand], |sema, [re]| {
        let ty = qualif.id.resolve_with(sema);
        if !ty.is_void() {
            let from = re.casted_ty().id.resolve_with(sema);
            if !ty.is_scalar(sema) {
                return Err(Diagnostic::CastToNonScalar);
            }
            if !from.is_scalar(sema) {
                return Err(Diagnostic::CastOfNonScalar);
            }
            if ty.is_pointer() != from.is_pointer() && (ty.is_floating() || from.is_floating()) {
                return Err(Diagnostic::InvalidCast(re.casted_ty(), qualif));
            }
            if !is_null
                && ty.is_pointer()
                && from.is_pointer()
                && is_pointer_to_function(sema, re.casted_ty()) != is_pointer_to_function(sema, qualif)
            {
                return Err(Diagnostic::InvalidCast(re.casted_ty(), qualif));
            }
            cast::convert(sema, re, qualif.id, is_null);
        }
        Ok((qualif, RValue))
    })
}

fn is_pointer_to_function(sema: &Sema, qty: QualifiedType) -> bool {
    matches!(qty.id.resolve_with(sema), ResolvedType::Pointer(inner) if inner.is_function(sema))
}
