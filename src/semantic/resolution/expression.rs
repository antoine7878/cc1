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

fn puts(sema: &mut Sema, e1: &ExpressionNode, re1: ResolvedExpression, e2: &ExpressionNode, re2: ResolvedExpression) {
    put(sema, e1, re1);
    put(sema, e2, re2);
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
    puts(sema, e1, lhs, e2, rhs);
    out.map(|q| (q, kind))
}

fn assign_target(lhs: &ResolvedExpression) -> Result<(), Diagnosis> {
    if lhs.kind == ExpressionKind::RValue {
        return Err(Diagnosis::AssignToRValue);
    }
    if lhs.ty.is_const {
        return Err(Diagnosis::ConstAssignement);
    }
    Ok(())
}

fn with_assignment<F>(
    sema: &mut Sema,
    ctx: &Context,
    lhs_node: &ExpressionNode,
    rhs_node: &ExpressionNode,
    f: F,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression, &mut ResolvedExpression, bool) -> Result<QualifiedType, Diagnosis>,
{
    let is_null = is_null_pointer_constant(sema, ctx, rhs_node);
    let (mut lhs, mut rhs) = takes(sema, lhs_node, rhs_node)?;
    let out = assign_target(&lhs).and_then(|()| {
        cast::lvalue_conversion(sema, &mut rhs, &rhs_node.span);
        f(sema, &mut lhs, &mut rhs, is_null)
    });
    puts(sema, e1, lhs, e2, rhs);
    out.map(|q| (q, ExpressionKind::RValue))
}

fn as_written<'a>(sema: &'a Sema, node: &ExpressionNode) -> Result<&'a ResolvedExpression, Diagnosis> {
    sema.expressions
        .get(&node.id)
        .and_then(Option::as_ref)
        .ok_or(Diagnosis::Poisoned)
}

fn check_is_arithmetic(sema: &Sema, ty: ResolvedTypeId) -> Result<(), Diagnosis> {
    match ty.resolve(sema).is_arithmetic(sema) {
        false => Err(Diagnosis::InvalidOperand),
        true => Ok(()),
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

fn additive(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    op: char,
) -> Result<QualifiedType, Diagnosis> {
    match (op, lhs.casted_ty().id.resolve(sema), rhs.casted_ty().id.resolve(sema)) {
        (_, l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => {
            cast::usual_arithmetic(sema, lhs, rhs);
            Ok(lhs.casted_ty())
        }
        (_, ResolvedType::Pointer(_), o) if o.is_integral(sema) => cast::pointer_integer_arithmetic(sema, lhs, rhs),
        ('+', o, ResolvedType::Pointer(_)) if o.is_integral(sema) => cast::pointer_integer_arithmetic(sema, rhs, lhs),
        ('-', ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => cast::pointer_minus_pointer(sema, lhs, rhs),
        _ => Err(Diagnosis::InvalidOperand),
    }
}

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
        Expression::Add(e1, e2) => with_operands(sema, e1, e2, RValue, |sema, lhs, rhs| additive(sema, lhs, rhs, '+')),
        Expression::Sub(e1, e2) => with_operands(sema, e1, e2, RValue, |sema, lhs, rhs| additive(sema, lhs, rhs, '-')),
        Expression::Minus(e) => with_operand(sema, e, RValue, |sema, re| {
            check_is_arithmetic(sema, re.casted_ty().id)?;
            cast::promote(sema, re);
            Ok(re.casted_ty())
        }),
        Expression::Cast(ty_node, operand) => {
            let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &node.span);
            let (qualif, _) =
                declaration::declared_type(sema, ctx, base, &ty_node.declarator).ok_or(Diagnosis::Poisoned)?;
            let is_null = is_null_pointer_constant(sema, ctx, operand);
            with_operand(sema, operand, RValue, |sema, re| {
                let ty = qualif.id.resolve(sema);
                if !matches!(ty, ResolvedType::Void) {
                    let from = re.casted_ty().id.resolve(sema);
                    if !ty.is_scalar(sema) || !from.is_scalar(sema) {
                        return Err(Diagnosis::CastToNonScalar);
                    }
                    if ty.is_pointer() != from.is_pointer() && (ty.is_floating() || from.is_floating()) {
                        return Err(Diagnosis::InvalidOperand);
                    }
                    cast::convert(sema, re, qualif.id, is_null);
                }
                Ok(qualif)
            })
        }
        Expression::Assign(e1, e2) => with_assignment(sema, ctx, e1, e2, |sema, lhs, rhs, is_null| {
            cast::assignment_conversion(sema, lhs, rhs, is_null)
        }),
        _ => Err(Diagnosis::Poisoned),
    }
}
