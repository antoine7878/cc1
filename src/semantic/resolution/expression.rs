use crate::ast::{Expression, ExpressionNode};
use crate::parser::{Context, Span};
use crate::semantic::model::cast::{self};
use crate::semantic::{
    Diagnosis, DiagnosisNode, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, ResolvedTypeId, Sema,
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
    let mut lhs = take(sema, e1)?;
    let mut rhs = match take(sema, e2) {
        Ok(rhs) => rhs,
        Err(diag) => {
            put(sema, e1, lhs);
            return Err(diag);
        }
    };
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
    if !sema.types.get(ty).is_arithmetic(&sema.tags) {
        return Err(Diagnosis::InvalidOperand);
    }
    Ok(())
}

fn pointer_integer_arithmetic(
    sema: &mut Sema,
    pointer: &mut ResolvedExpression,
    intergral: &mut ResolvedExpression,
    span: &Span,
) -> Result<QualifiedType, Diagnosis> {
    let ResolvedType::Pointer(inner) = sema.types.get(pointer.casted_ty().ty) else {
        return Err(Diagnosis::Poisoned);
    };
    match sema.types.get(inner.ty) {
        t if !t.is_complete(&sema.tags) => Err(Diagnosis::InvalidOperand),
        ResolvedType::Function { .. } => Err(Diagnosis::InvalidOperand),
        _ => {
            cast::promote(sema, intergral);
            Ok(pointer.casted_ty())
        }
    }
}

fn type_of(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    use ExpressionKind::{LValue, RValue};

    match sema.expressions.get(&node.id) {
        Some(Some(re)) => return Ok((re.ty, re.kind)),
        Some(None) => return Err(Diagnosis::Poisoned),
        None => (),
    }
    match node.id.resolve(ctx) {
        Expression::Identifier(_) => {
            let id = sema
                .bindings
                .get(&node.id)
                .copied()
                .flatten()
                .ok_or(Diagnosis::Poisoned)?;
            let sym = sema.symbols.get(id);
            Ok((sym.ty.ok_or(Diagnosis::Poisoned)?, sym.expression_kind()))
        }
        Expression::Constant(value) => Ok((value.ty(sema), RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema, ctx), LValue)),
        Expression::ConstantExpression(expr) => as_written(sema, expr).map(|re| (re.ty, re.kind)),
        Expression::Add(e1, e2) => with_operands(sema, e1, e2, RValue, |sema, lhs, rhs| {
            // lvalue_conversion(sema, lhs, &e1.span);
            // lvalue_conversion(sema, rhs, &e2.span);
            match (sema.types.get(lhs.casted_ty().ty), sema.types.get(rhs.casted_ty().ty)) {
                (l, r) if l.is_arithmetic(&sema.tags) && r.is_arithmetic(&sema.tags) => {
                    Ok(cast::usual_arithmetic(sema, lhs, rhs).casted_ty())
                }
                (ResolvedType::Pointer(_), o) if o.is_integral(&sema.tags) => {
                    pointer_integer_arithmetic(sema, lhs, rhs, &e1.span)
                }
                (o, ResolvedType::Pointer(_)) if o.is_integral(&sema.tags) => {
                    pointer_integer_arithmetic(sema, rhs, lhs, &e2.span)
                }
                _ => Err(Diagnosis::InvalidOperand),
            }
        }),
        Expression::Minus(e) => with_operand(sema, e, RValue, |sema, re| {
            // 6.3.3.3 The operand of the unary - operator shall have arithmetic type.
            check_is_arithmetic(sema, re.casted_ty().ty)?;
            cast::promote(sema, re);
            Ok(re.casted_ty())
        }),
        _ => Err(Diagnosis::Poisoned),
    }
}
