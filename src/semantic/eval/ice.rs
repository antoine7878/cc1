use std::cmp::Ordering;

use crate::arena::ResolveWith;
use crate::ast::visit::Visitor;
use crate::ast::{BinaryOp, Expression, ExpressionNode, Fold, Tag, UnaryOp, Value};
use crate::context::Context;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, QualifiedType, ResolvedType, Sema, SymbolKind, SymbolResolver,
};

struct DiagSink<'a>(&'a mut Vec<DiagnosisNode>);

impl DiagCollector for DiagSink<'_> {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        self.0
    }
}

pub fn eval_constant(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if sema.expr_consts.seen(expr.id) {
        return sema.expr_consts.get(expr.id).copied();
    }
    SymbolResolver::new(sema).visit_expression(ctx, expr);
    let mut collected = Vec::new();
    let folded = fold(sema, ctx, expr, &mut DiagSink(&mut collected));
    sema.diagnosis.append(&mut collected);
    let value = match folded {
        Ok(value) => Some(value),
        Err(diagnosis) => sema.add_diag(Diag::err(None, diagnosis), &expr.span),
    };
    sema.expr_consts.set(expr.id, value);
    value
}

pub fn try_fold(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if sema.expr_consts.seen(expr.id) {
        return sema.expr_consts.get(expr.id).copied();
    }
    fold(sema, ctx, expr, &mut DiagSink(&mut Vec::new())).ok()
}

fn scalar_ty(sema: &Sema, qty: QualifiedType) -> ResolvedType {
    let ty = qty.id.resolve(sema);
    if let ResolvedType::Tag(id) = ty
        && (*id).resolve(sema).kind == Tag::Enum
    {
        return ResolvedType::Int;
    }
    ty.clone()
}

fn node_ty(sema: &Sema, expr: &ExpressionNode) -> Result<QualifiedType, Diagnosis> {
    sema.expr_types.get(expr.id).map(|re| re.ty).ok_or(Diagnosis::Poisoned)
}

fn operand(sema: &mut Sema, ctx: &Context, e: &ExpressionNode, sink: &mut DiagSink) -> Result<Value, Diagnosis> {
    let value = fold(sema, ctx, e, sink)?;
    let casted = sema
        .expr_types
        .get(e.id)
        .map(|re| re.casted_ty())
        .ok_or(Diagnosis::Poisoned)?;
    let ty = scalar_ty(sema, casted);
    Fold::new(&sema.target)
        .convert(&ty, value)
        .ok_or(Diagnosis::NonIntegerConstantExpression)
}

fn divisor(sema: &Sema, ty: &ResolvedType, lhs: Value, rhs: Value, op: BinaryOp) -> Result<(), Diagnosis> {
    if rhs.is_zero() {
        return match op {
            BinaryOp::Div => Err(Diagnosis::DivisionByZero),
            BinaryOp::Mod => Err(Diagnosis::ModuloByZero),
            _ => unreachable!(),
        };
    }
    if rhs.to_i64() == -1 && Fold::new(&sema.target).is_min(ty, lhs) {
        return Err(Diagnosis::ConstantOverflow);
    }
    Ok(())
}

fn fold(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode, sink: &mut DiagSink) -> Result<Value, Diagnosis> {
    if let Some(value) = sema.expr_consts.get(expr.id) {
        return Ok(*value);
    }
    match expr.id.resolve(ctx) {
        Expression::ConstantExpression(inner) => fold(sema, ctx, inner, sink),
        Expression::Constant(value_node) => Ok(value_node.value),
        Expression::Identifier(_) => identifier(sema, expr),
        Expression::Unary(op, e) => unary_op(sema, ctx, expr, *op, e, sink),
        Expression::Binary(op, e1, e2) => binary_op(sema, ctx, expr, *op, e1, e2, sink),
        Expression::Ternary(condition, e1, e2) => conditional(sema, ctx, condition, e1, e2, sink),
        Expression::Cast(_, e) => cast(sema, ctx, expr, e, sink),
        Expression::SizeofExpr(_) | Expression::SizeofType(_) => Err(Diagnosis::Poisoned),
        Expression::StringLiteral(_)
        | Expression::Assign(_, _, _)
        | Expression::List(_)
        | Expression::ArraySubscripting(_, _)
        | Expression::FunctionCall(_, _)
        | Expression::Member(_, _, _) => Err(Diagnosis::NonConstantExpression),
    }
}

fn identifier(sema: &Sema, expr: &ExpressionNode) -> Result<Value, Diagnosis> {
    let id = sema.expr_bindings.get(expr.id).copied().ok_or(Diagnosis::NonConstantExpression)?;
    let symbol = id.resolve(sema);
    if symbol.kind != SymbolKind::Variant {
        return Err(Diagnosis::NonConstantExpression);
    }
    symbol.value.map(Value::Int).ok_or(Diagnosis::NonConstantExpression)
}

fn unary_op(
    sema: &mut Sema,
    ctx: &Context,
    expr: &ExpressionNode,
    op: UnaryOp,
    e: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    match op {
        UnaryOp::Plus => operand(sema, ctx, e, sink),
        UnaryOp::Minus | UnaryOp::BitNot => {
            let value = operand(sema, ctx, e, sink)?;
            let ty = scalar_ty(sema, node_ty(sema, expr)?);
            let folded = Fold::new(&sema.target).unary(&ty, op, value);
            Ok(sink.add_diag(folded, &expr.span))
        }
        UnaryOp::LogicalNot => Ok(operand(sema, ctx, e, sink)?.logical_not()),
        UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::Addr | UnaryOp::Deref => {
            Err(Diagnosis::NonConstantExpression)
        }
    }
}

fn binary_op(
    sema: &mut Sema,
    ctx: &Context,
    expr: &ExpressionNode,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    match op {
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => logical(sema, ctx, op, e1, e2, sink),
        BinaryOp::Greater
        | BinaryOp::Lower
        | BinaryOp::GreaterEq
        | BinaryOp::LowerEq
        | BinaryOp::Eq
        | BinaryOp::Neq => comparison(sema, ctx, op, e1, e2, sink),
        BinaryOp::Div | BinaryOp::Mod => divide(sema, ctx, expr, op, e1, e2, sink),
        _ => arithmetic(sema, ctx, expr, op, e1, e2, sink),
    }
}

fn logical(
    sema: &mut Sema,
    ctx: &Context,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    let lhs = fold(sema, ctx, e1, sink)?.is_true();
    let value = match op {
        BinaryOp::LogicalAnd => lhs && fold(sema, ctx, e2, sink)?.is_true(),
        BinaryOp::LogicalOr => lhs || fold(sema, ctx, e2, sink)?.is_true(),
        _ => unreachable!(),
    };
    Ok(Value::from(value))
}

fn comparison(
    sema: &mut Sema,
    ctx: &Context,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    let lhs = operand(sema, ctx, e1, sink)?;
    let rhs = operand(sema, ctx, e2, sink)?;
    let ordering = Fold::new(&sema.target).compare(lhs, rhs);
    let holds = match op {
        BinaryOp::Greater => matches!(ordering, Some(Ordering::Greater)),
        BinaryOp::Lower => matches!(ordering, Some(Ordering::Less)),
        BinaryOp::GreaterEq => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
        BinaryOp::LowerEq => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
        BinaryOp::Eq => matches!(ordering, Some(Ordering::Equal)),
        BinaryOp::Neq => matches!(ordering, Some(Ordering::Less | Ordering::Greater)),
        _ => unreachable!(),
    };
    Ok(Value::from(holds))
}

fn divide(
    sema: &mut Sema,
    ctx: &Context,
    expr: &ExpressionNode,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    let lhs = operand(sema, ctx, e1, sink)?;
    let rhs = operand(sema, ctx, e2, sink)?;
    let ty = scalar_ty(sema, node_ty(sema, expr)?);
    divisor(sema, &ty, lhs, rhs, op)?;
    Ok(Fold::new(&sema.target).binary(&ty, op, lhs, rhs).res)
}

fn arithmetic(
    sema: &mut Sema,
    ctx: &Context,
    expr: &ExpressionNode,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    let lhs = operand(sema, ctx, e1, sink)?;
    let rhs = operand(sema, ctx, e2, sink)?;
    let ty = scalar_ty(sema, node_ty(sema, expr)?);
    let folded = Fold::new(&sema.target).binary(&ty, op, lhs, rhs);
    Ok(sink.add_diag(folded, &expr.span))
}

fn conditional(
    sema: &mut Sema,
    ctx: &Context,
    condition: &ExpressionNode,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    match fold(sema, ctx, condition, sink)?.is_true() {
        true => operand(sema, ctx, e1, sink),
        false => operand(sema, ctx, e2, sink),
    }
}

fn cast(
    sema: &mut Sema,
    ctx: &Context,
    expr: &ExpressionNode,
    e: &ExpressionNode,
    sink: &mut DiagSink,
) -> Result<Value, Diagnosis> {
    if node_ty(sema, expr)?.is_void(sema) {
        return Err(Diagnosis::NonIntegerConstantExpression);
    }
    let operand_ty = node_ty(sema, e)?;
    if operand_ty.is_floating(sema) && !matches!(e.id.resolve(ctx), Expression::Constant(_)) {
        return Err(Diagnosis::NonConstantExpression);
    }
    operand(sema, ctx, e, sink)
}
