use crate::arena::OptionPoisoned;
use crate::ast::{ExpressionNode, Type, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::resolution::expression::*;
use crate::semantic::{QualifiedType, Sema, constrain, declaration, layout};

pub fn size_of_e(sema: &mut Sema, node: &ExpressionNode, e: &ExpressionNode) -> R {
    let (ty, is_bit_field) = with_ops(&mut *sema, [e], |sema, [re]| {
        Ok((re.ty, is_bit_field(sema, sema.expr_bindings.get(e.id).copied())))
    })?;
    let result = size_t(sema, ty, is_bit_field)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
}

pub fn size_of_ty(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, ty: &Type, span: &Span) -> R {
    let base = declaration::base_type(sema, ctx, &ty.specifiers, span);
    let (ty, _) = declaration::declared_type(sema, ctx, base, &ty.declarator).ok_poisoned()?;
    let result = size_t(sema, ty, false)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
}

fn size_t(sema: &Sema, ty: QualifiedType, is_bit_field: bool) -> R {
    constrain::expression::check_sizeof(
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
        && let Some(value) = sema
            .target
            .cast(&sema.target.size_t, Value::UnsignedLong(layout.size.into()))
    {
        sema.expr_consts.set(node.id, Some(value));
    }
}
