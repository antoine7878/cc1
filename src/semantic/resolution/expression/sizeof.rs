use crate::arena::OptionPoisoned;
use crate::ast::{ExpressionNode, Type, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::ExpressionKind::RValue;
use crate::semantic::{Diagnosis, QualifiedType, Sema, declaration, layout};

use super::operand::{R, is_bit_field, with_ops};

pub(super) fn size_of_e(sema: &mut Sema, node: &ExpressionNode, e: &ExpressionNode) -> R {
    let ty = with_ops(&mut *sema, [e], |sema, [re]| {
        if is_bit_field(sema, sema.expr_bindings.get(e.id).copied()) {
            return Err(Diagnosis::SizeofBitfield);
        }
        Ok(re.ty)
    })?;
    let result = size_t(sema, ty)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
}

pub(super) fn size_of_ty(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, ty: &Type, span: &Span) -> R {
    let base = declaration::base_type(sema, ctx, &ty.specifiers, span);
    let (ty, _) = declaration::declared_type(sema, ctx, base, &ty.declarator).ok_poisoned()?;
    let result = size_t(sema, ty)?;
    set_sizeof_constant(sema, node, ty);
    Ok(result)
}

fn size_t(sema: &Sema, ty: QualifiedType) -> R {
    if ty.is_void(sema) {
        return Err(Diagnosis::SizeofVoid);
    }
    if ty.is_function(sema) {
        return Err(Diagnosis::SizeofFunction);
    }
    if !ty.is_complete(sema) {
        return Err(Diagnosis::SizeofIncomplete(ty));
    }
    let qty = QualifiedType::new(sema.builtins.size_t, false, false);
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
