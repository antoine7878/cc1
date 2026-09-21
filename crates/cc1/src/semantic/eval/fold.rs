use std::cmp::Ordering;

use crate::ast::{BinaryOp, ConstFolder, ConstValue, Expression, ExpressionNode, Tag, UnaryOp};
use crate::semantic::{
    Diag, Diagnostic, DiagnosticNode, DiagnosticSink, QualifiedType, ResolvedType, Sema, SymbolKind,
};

struct VecSink<'a>(&'a mut Vec<DiagnosticNode>);

impl DiagnosticSink for VecSink<'_> {
    fn diagnostics(&mut self) -> &mut Vec<DiagnosticNode> {
        self.0
    }
}

/// Folds an already resolved expression. Binding happens in the resolution pass, so a caller that
/// is still binding must go through `Resolver::eval_constant`.
pub fn eval_constant(sema: &mut Sema, expr: &ExpressionNode) -> Option<ConstValue> {
    if sema.expr_consts.contains(expr.id) {
        return sema.expr_consts.get(expr.id).copied();
    }
    let mut collected = Vec::new();
    let folded = evaluate(sema, expr, &mut VecSink(&mut collected));
    sema.diagnostics.append(&mut collected);
    let value = match folded {
        Ok(value) => Some(value),
        Err(diagnostic) => sema.add_diag(Diag::err(None, diagnostic), &expr.span),
    };
    sema.expr_consts.set(expr.id, value);
    value
}

pub fn try_fold(sema: &mut Sema, expr: &ExpressionNode) -> Option<ConstValue> {
    if sema.expr_consts.contains(expr.id) {
        return sema.expr_consts.get(expr.id).copied();
    }
    evaluate(sema, expr, &mut VecSink(&mut Vec::new())).ok()
}

/// 6.4 Each constant expression shall evaluate to a constant that is in the range of representable
/// values for its type: an initializer that folds keeps the overflow diagnostics of its evaluation,
/// and a floating value that does not fit the integer object it initializes is an overflow too.
pub fn fold_initializer(sema: &mut Sema, ty: QualifiedType, expr: &ExpressionNode) -> Option<ConstValue> {
    let mut collected = Vec::new();
    let value = evaluate(sema, expr, &mut VecSink(&mut collected)).ok()?;
    sema.diagnostics.append(&mut collected);
    if !fits(&scalar_ty(sema, ty), value) {
        sema.add_diag(Diag::err((), Diagnostic::ConstantOverflow), &expr.span);
    }
    Some(value)
}

fn fits(ty: &ResolvedType, value: ConstValue) -> bool {
    let (Some(min), Some(max)) = (ty.min_value(), ty.max_value()) else { return true };
    if !value.is_floating() {
        return true;
    }
    let v = value.to_f64();
    !v.is_nan() && v > min as f64 - 1.0 && v < max as f64 + 1.0
}

fn scalar_ty(sema: &Sema, qty: QualifiedType) -> ResolvedType {
    let ty = qty.id.resolve_with(sema);
    if let ResolvedType::Tag(id) = ty
        && (*id).resolve_with(sema).kind == Tag::Enum
    {
        return ResolvedType::Int;
    }
    ty.clone()
}

fn node_ty(sema: &Sema, expr: &ExpressionNode) -> Result<QualifiedType, Diagnostic> {
    sema.expressions.get(expr.id).map(|re| re.ty).ok_or(Diagnostic::Poisoned)
}

fn operand(sema: &mut Sema, e: &ExpressionNode, sink: &mut VecSink) -> Result<ConstValue, Diagnostic> {
    let value = evaluate(sema, e, sink)?;
    let casted = sema.expressions.get(e.id).map(|re| re.casted_ty()).ok_or(Diagnostic::Poisoned)?;
    let ty = scalar_ty(sema, casted);
    ConstFolder.convert(&ty, value).ok_or(Diagnostic::NonIntegerConstantExpression)
}

fn divisor(ty: &ResolvedType, lhs: ConstValue, rhs: ConstValue, op: BinaryOp) -> Result<(), Diagnostic> {
    if rhs.is_zero() {
        return match op {
            BinaryOp::Div => Err(Diagnostic::DivisionByZero),
            BinaryOp::Mod => Err(Diagnostic::ModuloByZero),
            _ => unreachable!(),
        };
    }
    if rhs.to_i64() == -1 && ConstFolder.is_min(ty, lhs) {
        return Err(Diagnostic::ConstantOverflow);
    }
    Ok(())
}

fn evaluate(sema: &mut Sema, expr: &ExpressionNode, sink: &mut VecSink) -> Result<ConstValue, Diagnostic> {
    if let Some(value) = sema.expr_consts.get(expr.id) {
        return Ok(*value);
    }
    if sema.expr_consts.poisoned(expr.id) {
        return Err(Diagnostic::Poisoned);
    }
    match expr.id.resolve() {
        Expression::ConstantExpression(inner) => {
            integral_operands(sema, inner)?;
            evaluate(sema, inner, sink)
        }
        Expression::Constant(value_node) => Ok(value_node.value),
        Expression::Identifier(_) => identifier(sema, expr),
        Expression::Unary(op, e) => unary_op(sema, expr, *op, e, sink),
        Expression::Binary(op, e1, e2) => binary_op(sema, expr, *op, e1, e2, sink),
        Expression::Ternary(condition, e1, e2) => conditional(sema, condition, e1, e2, sink),
        Expression::Cast(_, e) => cast(sema, expr, e, sink),
        Expression::SizeofExpr(_) | Expression::SizeofType(_) => Err(Diagnostic::Poisoned),
        Expression::StringLiteral(_)
        | Expression::Assign(_, _, _)
        | Expression::List(_)
        | Expression::ArraySubscripting(_, _)
        | Expression::FunctionCall(_, _)
        | Expression::Member(_, _, _) => Err(Diagnostic::NonConstantExpression),
    }
}

fn integral_operands(sema: &Sema, expr: &ExpressionNode) -> Result<(), Diagnostic> {
    let operands: Vec<&ExpressionNode> = match expr.id.resolve() {
        Expression::Cast(_, _) | Expression::SizeofExpr(_) | Expression::SizeofType(_) => return Ok(()),
        Expression::ConstantExpression(e) | Expression::Unary(_, e) => vec![e],
        Expression::Binary(_, e1, e2) => vec![e1, e2],
        Expression::Ternary(condition, e1, e2) => vec![condition, e1, e2],
        _ => Vec::new(),
    };
    for operand in operands {
        if node_ty(sema, operand)?.is_floating(sema) {
            return Err(Diagnostic::NonIntegerConstantExpression);
        }
        integral_operands(sema, operand)?;
    }
    Ok(())
}

fn identifier(sema: &Sema, expr: &ExpressionNode) -> Result<ConstValue, Diagnostic> {
    let id = sema.expr_bindings.get(expr.id).copied().ok_or(Diagnostic::NonConstantExpression)?;
    let symbol = id.resolve_with(sema);
    if symbol.kind != SymbolKind::Enumerator {
        return Err(Diagnostic::NonConstantExpression);
    }
    symbol.value.map(ConstValue::Int).ok_or(Diagnostic::NonConstantExpression)
}

fn unary_op(
    sema: &mut Sema,
    expr: &ExpressionNode,
    op: UnaryOp,
    e: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    match op {
        UnaryOp::Plus => operand(sema, e, sink),
        UnaryOp::Minus | UnaryOp::BitNot => {
            let value = operand(sema, e, sink)?;
            let ty = scalar_ty(sema, node_ty(sema, expr)?);
            let folded = ConstFolder.unary(&ty, op, value);
            Ok(sink.add_diag(folded, &expr.span))
        }
        UnaryOp::LogicalNot => Ok(operand(sema, e, sink)?.logical_not()),
        UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::Addr | UnaryOp::Deref => {
            Err(Diagnostic::NonConstantExpression)
        }
    }
}

fn binary_op(
    sema: &mut Sema,
    expr: &ExpressionNode,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    match op {
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => logical(sema, op, e1, e2, sink),
        BinaryOp::Greater
        | BinaryOp::Lower
        | BinaryOp::GreaterEq
        | BinaryOp::LowerEq
        | BinaryOp::Eq
        | BinaryOp::Neq => comparison(sema, op, e1, e2, sink),
        BinaryOp::Div | BinaryOp::Mod => divide(sema, expr, op, e1, e2, sink),
        _ => arithmetic(sema, expr, op, e1, e2, sink),
    }
}

fn logical(
    sema: &mut Sema,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    let lhs = evaluate(sema, e1, sink)?.is_true();
    let value = match op {
        BinaryOp::LogicalAnd => lhs && evaluate(sema, e2, sink)?.is_true(),
        BinaryOp::LogicalOr => lhs || evaluate(sema, e2, sink)?.is_true(),
        _ => unreachable!(),
    };
    Ok(ConstValue::from(value))
}

fn comparison(
    sema: &mut Sema,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    let lhs = operand(sema, e1, sink)?;
    let rhs = operand(sema, e2, sink)?;
    let ordering = ConstFolder.compare(lhs, rhs);
    let holds = match op {
        BinaryOp::Greater => matches!(ordering, Some(Ordering::Greater)),
        BinaryOp::Lower => matches!(ordering, Some(Ordering::Less)),
        BinaryOp::GreaterEq => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
        BinaryOp::LowerEq => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
        BinaryOp::Eq => matches!(ordering, Some(Ordering::Equal)),
        BinaryOp::Neq => matches!(ordering, Some(Ordering::Less | Ordering::Greater)),
        _ => unreachable!(),
    };
    Ok(ConstValue::from(holds))
}

fn divide(
    sema: &mut Sema,
    expr: &ExpressionNode,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    let lhs = operand(sema, e1, sink)?;
    let rhs = operand(sema, e2, sink)?;
    let ty = scalar_ty(sema, node_ty(sema, expr)?);
    divisor(&ty, lhs, rhs, op)?;
    Ok(ConstFolder.binary(&ty, op, lhs, rhs).res)
}

fn arithmetic(
    sema: &mut Sema,
    expr: &ExpressionNode,
    op: BinaryOp,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    let lhs = operand(sema, e1, sink)?;
    let rhs = operand(sema, e2, sink)?;
    let ty = scalar_ty(sema, node_ty(sema, expr)?);
    let folded = ConstFolder.binary(&ty, op, lhs, rhs);
    Ok(sink.add_diag(folded, &expr.span))
}

fn conditional(
    sema: &mut Sema,
    condition: &ExpressionNode,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    match evaluate(sema, condition, sink)?.is_true() {
        true => operand(sema, e1, sink),
        false => operand(sema, e2, sink),
    }
}

fn cast(
    sema: &mut Sema,
    expr: &ExpressionNode,
    e: &ExpressionNode,
    sink: &mut VecSink,
) -> Result<ConstValue, Diagnostic> {
    if node_ty(sema, expr)?.is_void(sema) {
        return Err(Diagnostic::NonIntegerConstantExpression);
    }
    let operand_ty = node_ty(sema, e)?;
    if operand_ty.is_floating(sema) && !matches!(e.id.resolve(), Expression::Constant(_)) {
        return Err(Diagnostic::NonConstantExpression);
    }
    operand(sema, e, sink)
}
