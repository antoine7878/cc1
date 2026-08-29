use crate::ast::{Expression, ExpressionNode};
use crate::parser::Context;
use crate::semantic::model::cast;
use crate::semantic::{Diagnosis, DiagnosisNode, ExpressionKind, QualifiedType, ResolvedExpression, Sema};

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

fn with_operand<F>(sema: &mut Sema, node: &ExpressionNode, f: F) -> Result<QualifiedType, Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression) -> Result<QualifiedType, Diagnosis>,
{
    let mut re = take(sema, node)?;
    cast::lvalue_conversion(sema, &mut re, &node.span);
    let out = f(sema, &mut re);
    put(sema, node, re);
    out
}

fn with_operands<F>(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode, f: F) -> Result<QualifiedType, Diagnosis>
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
    out
}

fn as_written<'a>(sema: &'a Sema, node: &ExpressionNode) -> Result<&'a ResolvedExpression, Diagnosis> {
    sema.expressions
        .get(&node.id)
        .and_then(Option::as_ref)
        .ok_or(Diagnosis::Poisoned)
}

fn type_of(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
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
                .ok_or(Diagnosis::NonConstantExpression)?;
            Ok((sema.symbols.get(id).ty.unwrap(), ExpressionKind::RValue))
        }
        Expression::Constant(value) => Ok((value.ty(sema), ExpressionKind::RValue)),
        Expression::StringLiteral(value) => Ok((value.ty(sema, ctx), ExpressionKind::RValue)),
        Expression::ConstantExpression(expr) => type_of(sema, ctx, expr),
        Expression::Add(_e1, _e2) => Err(Diagnosis::Poisoned),
        _ => todo!(),
    }
}
