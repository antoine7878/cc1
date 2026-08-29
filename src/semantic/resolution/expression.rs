use crate::ast::{Expression, ExpressionNode};
use crate::parser::Context;
use crate::semantic::model::cast;
use crate::semantic::{Diagnosis, DiagnosisNode, ExpressionKind, QualifiedType, ResolvedExpression, Sema};

pub fn run(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) {
    let ty = type_of(sema, ctx, node);
    match ty {
        Ok((ty, kind)) => {
            sema.expressions
                .entry(node.id)
                .or_insert(ResolvedExpression::new(ty, kind))
                .ty = ty
        }
        Err(diag) => sema.diagnosis.push(DiagnosisNode {
            span: node.span,
            inner: diag,
        }),
    }
}

fn with_operand<F>(sema: &mut Sema, node: &ExpressionNode, f: F) -> Result<QualifiedType, Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression) -> Result<QualifiedType, Diagnosis>,
{
    let mut re = sema.expressions.remove(&node.id).ok_or(Diagnosis::Poisoned)?;
    cast::lvalue_conversion(sema, &mut re, &node.span);
    let out = f(sema, &mut re);
    sema.expressions.insert(node.id, re);
    out
}

fn with_operands<F>(sema: &mut Sema, e1: &ExpressionNode, e2: &ExpressionNode, f: F) -> Result<QualifiedType, Diagnosis>
where
    F: FnOnce(&mut Sema, &mut ResolvedExpression, &mut ResolvedExpression) -> Result<QualifiedType, Diagnosis>,
{
    let mut lhs = sema.expressions.remove(&e1.id).ok_or(Diagnosis::Poisoned)?;
    let mut rhs = match sema.expressions.remove(&e2.id) {
        Some(rhs) => rhs,
        None => {
            sema.expressions.insert(e1.id, lhs);
            return Err(Diagnosis::Poisoned);
        }
    };
    cast::lvalue_conversion(sema, &mut lhs, &e1.span);
    cast::lvalue_conversion(sema, &mut rhs, &e2.span);
    let out = f(sema, &mut lhs, &mut rhs);
    sema.expressions.insert(e1.id, lhs);
    sema.expressions.insert(e2.id, rhs);
    out
}

fn as_written<'a>(sema: &'a Sema, node: &ExpressionNode) -> Result<&'a ResolvedExpression, Diagnosis> {
    sema.expressions.get(&node.id).ok_or(Diagnosis::Poisoned)
}

fn type_of(
    sema: &mut Sema,
    ctx: &Context,
    node: &ExpressionNode,
) -> Result<(QualifiedType, ExpressionKind), Diagnosis> {
    if let Some(re) = sema.expressions.get(&node.id) {
        return Ok((re.ty, re.kind));
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
