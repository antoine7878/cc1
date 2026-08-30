use crate::arena::ResolveWith;
use crate::ast::{Expression, ExpressionNode};
use crate::context::Context;
use crate::semantic::ice::try_fold;
use crate::semantic::model::cast;
use crate::semantic::{
    Diagnosis, DiagnosisNode, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, ResolvedTypeId, Sema,
    declaration,
};

pub fn run(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) {
    if sema.expressions.contains_key(&node.id) {
        return;
    }
    let resolved = match type_of(sema, ctx, node) {
        Ok((ty, kind)) => Some(ResolvedExpression::new(ty, kind)),
        Err(Diagnosis::Poisoned) => None,
        Err(inner) => {
            sema.diagnosis.push(DiagnosisNode { span: node.span, inner });
            None
        }
    };
    sema.expressions.insert(node.id, resolved);
}

fn take(sema: &mut Sema, node: &ExpressionNode) -> Result<ResolvedExpression, Diagnosis> {
    let slot = sema.expressions.get_mut(&node.id).ok_or(Diagnosis::Poisoned)?;
    slot.take().ok_or(Diagnosis::Poisoned)
}

fn takes(
    sema: &mut Sema,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
) -> Result<(ResolvedExpression, ResolvedExpression), Diagnosis> {
    let lhs = take(sema, e1)?;
    let rhs = match take(sema, e2) {
        Ok(rhs) => rhs,
        Err(diag) => {
            put(sema, e1, lhs);
            return Err(diag);
        }
    };
    Ok((lhs, rhs))
}

fn put(sema: &mut Sema, node: &ExpressionNode, re: ResolvedExpression) {
    sema.expressions.insert(node.id, Some(re));
}

fn with_operand<F>(
    sema: &mut Sema,
    node: &ExpressionNode,
    kind: ExpressionKind,
    f: F,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression) -> Result<QualifiedType, Diagnosis>,
{
    let mut re = take(sema, node)?;
    cast::lvalue_conversion(sema, &mut re, &node.span);
    let out = f(sema, &mut re);
    put(sema, node, re);
    out.map(|q| (q, kind))
}

fn with_operands<F>(
    sema: &mut Sema,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    kind: ExpressionKind,
    f: F,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression, &mut ResolvedExpression) -> Result<QualifiedType, Diagnosis>,
{
    let (mut lhs, mut rhs) = takes(sema, e1, e2)?;
    cast::lvalue_conversion(sema, &mut lhs, &e1.span);
    cast::lvalue_conversion(sema, &mut rhs, &e2.span);
    let out = f(sema, &mut lhs, &mut rhs);
    put(sema, e1, lhs);
    put(sema, e2, rhs);
    out.map(|q| (q, kind))
}

fn as_written<'a>(sema: &'a Sema, node: &ExpressionNode) -> Result<&'a ResolvedExpression, Diagnosis> {
    sema.expressions
        .get(&node.id)
        .and_then(Option::as_ref)
        .ok_or(Diagnosis::Poisoned)
}

fn check_is_arithmetic(sema: &Sema, ty: ResolvedTypeId) -> Result<(), Diagnosis> {
    if !ty.resolve(sema).is_arithmetic(sema) {
        return Err(Diagnosis::InvalidOperand);
    }
    Ok(())
}

fn pointer_integer_arithmetic(
    sema: &mut Sema,
    pointer: &mut ResolvedExpression,
    intergral: &mut ResolvedExpression,
) -> Result<QualifiedType, Diagnosis> {
    let ResolvedType::Pointer(inner) = pointer.casted_ty().id.resolve(sema) else {
        return Err(Diagnosis::Poisoned);
    };
    match inner.id.resolve(sema) {
        t if !t.is_complete(&sema.tags) => Err(Diagnosis::InvalidOperand),
        ResolvedType::Function { .. } => Err(Diagnosis::InvalidOperand),
        _ => {
            cast::promote(sema, intergral);
            Ok(pointer.casted_ty())
        }
    }
}

fn is_null_pointer_constant(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> bool {
    let mut node = node;
    if let Expression::Cast(ty_node, op) = node.id.resolve(ctx) {
        let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
        let Some((qualif, _)) = declaration::declared_type(sema, ctx, base, &ty_node.declarator) else {
            return false;
        };
        if !matches!(qualif.id.resolve(sema), ResolvedType::Pointer(q) if q.id == sema.builtins.void)
            || qualif.is_const
            || qualif.is_volatile
        {
            return false;
        }

        node = op;
    }
    let Some(Some(re)) = sema.expressions.get(&node.id) else { return false };
    re.casted_ty().id.resolve(sema).is_integer() && try_fold(sema, ctx, node).is_some_and(|v| v.is_zero())
}

// fn assignation(
//     sema: &mut Sema,
//     ctx: &Context,
//     e1: &ExpressionNode,
//     e2: &ExpressionNode,
// ) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
//     let (mut lhs, mut rhs) = takes(sema, e1, e2)?;
//     if lhs.kind == ExpressionKind::RValue {
//         return Err(Diagnosis::AssignToRValue);
//     }
//     if lhs.ty.is_const {
//         return Err(Diagnosis::ConstAssignement);
//     }
//     cast::lvalue_conversion(sema, &mut rhs, &e2.span);
//
//     // let l_ty = sema.
//     // let lty = lhs.id.resolve(ctx);
//     // let rty = rhs.id.resolve(ctx);
//     // true
//
//     let out = Ok((lhs.ty, ExpressionKind::RValue));
//     put(sema, e1, lhs);
//     put(sema, e2, rhs);
//     out
// }

fn type_of(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    use ExpressionKind::{LValue, RValue};

    match node.id.resolve(ctx) {
        Expression::Identifier(_) => {
            let id = sema
                .bindings
                .get(&node.id)
                .copied()
                .flatten()
                .ok_or(Diagnosis::Poisoned)?;
            let sym = id.resolve(sema);
            Ok((sym.ty.ok_or(Diagnosis::Poisoned)?, sym.expression_kind()))
        }
        Expression::Constant(value) => Ok((value.ty(sema), RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema, ctx), LValue)),
        Expression::ConstantExpression(expr) => as_written(sema, expr).map(|re| (re.ty, re.kind)),
        Expression::Add(e1, e2) => with_operands(sema, e1, e2, RValue, |sema, lhs, rhs| {
            match (lhs.casted_ty().id.resolve(sema), rhs.casted_ty().id.resolve(sema)) {
                (l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => {
                    Ok(cast::usual_arithmetic(sema, lhs, rhs).casted_ty())
                }
                (ResolvedType::Pointer(_), o) if o.is_integral(sema) => pointer_integer_arithmetic(sema, lhs, rhs),
                (o, ResolvedType::Pointer(_)) if o.is_integral(sema) => pointer_integer_arithmetic(sema, rhs, lhs),
                _ => Err(Diagnosis::InvalidOperand),
            }
        }),
        Expression::Minus(e) => with_operand(sema, e, RValue, |sema, re| {
            // 6.3.3.3 The operand of the unary - operator shall have arithmetic type.
            check_is_arithmetic(sema, re.casted_ty().id)?;
            cast::promote(sema, re);
            Ok(re.casted_ty())
        }),
        Expression::Cast(ty_node, operand) => {
            // resolve the target type from the parenthesized type name
            let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
            let (qualif, _) =
                declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_or(Diagnosis::Poisoned)?;
            // Err((Diagnosis::Poisoned))

            //qualif's category
            with_operand(sema, operand, RValue, |sema, re| {
                // 6.3.4 Unless the type name specifies void type, the type name shall specify
                // qualified or unqualified scalar type and the operand shall have scalar type.
                let ty = qualif.id.resolve(sema);
                if !matches!(ty, ResolvedType::Void) {
                    if !ty.is_scalar(sema) || !re.casted_ty().id.resolve(sema).is_scalar(sema) {
                        return Err(Diagnosis::InvalidOperand);
                    }
                    let a = is_null_pointer_constant(sema, ctx, operand);
                    cast::convert(sema, re, qualif.id, a)
                }
                Ok(qualif)
            })
        }

        // Expression::Assign(e1, e2) => {}
        // Expression::Cast(ty_node, operand) => {
        //     let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
        //     let (to_qty, _) =
        //         declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_or(Diagnosis::CastToNonScalar)?;
        //     let Some(Some(from_re)) = sema.expressions.get(&operand.id) else {
        //         return Err(Diagnosis::Poisoned);
        //     };
        //     let to_ty = to_qty.id.resolve(sema);
        //     let from_ty = from_re.ty.id.resolve(sema);
        //     // 6.3.4
        //     // Unless the type name specifies void type, the type name shall specify qualified or unqualified scalar type
        //     if to_qty.id != sema.builtins.void && !to_ty.is_scalar(sema) {
        //         return Err(Diagnosis::CastToNonScalar);
        //     }
        //     // and the operand shall have scalar type.
        //     if !from_ty.is_scalar(sema) {
        //         return Err(Diagnosis::CastOfNonScalar);
        //     }
        //     // A pointer may be convened to an integral type. The size of integer required and the result
        //     // are implementation-defined If the space provided is not long enough. the behavior is undefined
        //     let p_to_int = from_ty.is_pointer() && to_ty.is_integral(sema);
        //     // An arbitary integer may be converted to a pointer. The result is implementation defined
        //     let int_to_p = from_ty.is_integral(sema) && to_ty.is_pointer();
        //     // A pointer to an object or incomplete type may be converted to a pointer to a different
        //     // object type or a different incomplete type. The resulting pointer might not be valid if it is
        //     // improperly aligned for the type pointed to.
        //     let p_to_p = from_ty.is_pointer() && to_ty.is_pointer();
        //     // A pointer to a function of one type may be converted to a pointer to a function of another
        //     // type and back again; the result shall compare equal to the original pointer. If a converted
        //     // pointer is used to call a function that has a type that is not compatible with the type of the
        //     // called function. the behavior is undefined.
        //     let f_to_f = from_ty.is_function() && to_ty.is_function();
        //
        //     Ok((to_qty, RValue))
        // }
        _ => Err(Diagnosis::Poisoned),
    }
}
