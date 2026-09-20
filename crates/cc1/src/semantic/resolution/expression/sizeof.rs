use libft::Span;

use crate::arena::OptionPoisoned;
use crate::ast::{ConstValue, ExpressionNode, TypeName};
use crate::semantic::ValueCategory::RValue;
use crate::semantic::resolution::expression::*;
use crate::semantic::{QualifiedType, Resolver, Sema, constraints, declaration, layout};

pub fn sizeof_expr(sema: &mut Sema, node: &ExpressionNode, e: &ExpressionNode) -> ExprResult {
    let (ty, is_bit_field) =
        with_ops(&mut *sema, [e], |sema, [re]| Ok((re.ty, is_bit_field(sema, sema.expr_bindings.get(e.id).copied()))))?;
    let result = sizeof_result(sema, ty, is_bit_field)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
}

pub fn sizeof_type(resolver: &mut Resolver, node: &ExpressionNode, ty: &TypeName, span: &Span) -> ExprResult {
    let base = declaration::base_type(resolver, &ty.specifiers, span);
    let (ty, _) = declaration::declared_type(resolver, base, &ty.declarator).ok_poisoned()?;
    let result = sizeof_result(resolver.sema, ty, false)?;
    set_sizeof_constant(resolver.sema, node, ty);
    Ok(result)
}

fn sizeof_result(sema: &Sema, ty: QualifiedType, is_bit_field: bool) -> ExprResult {
    constraints::expression::check_sizeof(
        is_bit_field,
        ty.is_void(sema),
        ty.is_function(sema),
        ty.is_complete(sema),
        ty,
    )
    .into_result()?;
    let qty = QualifiedType::plain(sema.builtins.size_t);
    Ok((qty, RValue))
}

fn set_sizeof_constant(sema: &mut Sema, node: &ExpressionNode, ty: QualifiedType) {
    if let Some(layout) = layout::of(sema, ty.id)
        && let Some(value) = sema.builtins.size_t.resolve_with(sema).cast(ConstValue::UnsignedLong(layout.size.into()))
    {
        sema.expr_consts.set(node.id, Some(value));
    }
}
