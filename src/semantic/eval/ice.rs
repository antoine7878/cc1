use crate::arena::ResolveWith;
use crate::ast::visit::Visitor;
use crate::ast::{BinaryOp, Expression, ExpressionNode, Fold, Tag, UnaryOp, Value};
use crate::context::Context;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, QualifiedType, ResolvedType, Sema, SymbolKind, SymbolResolver,
    declaration, layout,
};

struct Sink<'a>(&'a mut Vec<DiagnosisNode>);

impl DiagCollector for Sink<'_> {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        self.0
    }
}

pub fn eval_constant(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if let Some(cached) = sema.constant_cached(expr.id) {
        return cached;
    }
    SymbolResolver::new(sema).visit_expression(ctx, expr);
    let mut collected = Vec::new();
    let folded = fold(sema, ctx, expr, &mut Sink(&mut collected));
    sema.diagnosis.append(&mut collected);
    let value = match folded {
        Ok(value) => Some(value),
        Err(Diagnosis::Poisoned) => None, // already reported by the type pass
        Err(diagnosis) => sema.add_diag(Diag::err(None, diagnosis), &expr.span),
    };
    sema.set_constant(expr.id, value);
    value
}

pub fn try_fold(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if let Some(cached) = sema.constant_cached(expr.id) {
        return cached;
    }
    fold(sema, ctx, expr, &mut Sink(&mut Vec::new())).ok()
}

fn cast(sema: &mut Sema, qualif: QualifiedType, val: Value) -> Result<Value, Diagnosis> {
    let ty = qualif.id.resolve(sema);
    if let ResolvedType::Tag(id) = ty {
        return match (*id).resolve(sema).kind {
            Tag::Enum => sema
                .target
                .cast(&ResolvedType::Int, val)
                .ok_or(Diagnosis::NonIntegerConstantExpression),
            _ => Err(Diagnosis::CastToNonScalar),
        };
    }
    if let Some(casted) = sema.target.cast(ty, val) {
        return Ok(casted);
    }
    match ty {
        ResolvedType::Array { .. } | ResolvedType::Void => Err(Diagnosis::CastToNonScalar),
        _ => Err(Diagnosis::NonIntegerConstantExpression),
    }
}

fn operands(
    sema: &mut Sema,
    ctx: &Context,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
    sink: &mut Sink,
) -> Result<(Value, Value), Diagnosis> {
    let lhs = fold(sema, ctx, e1, sink)?;
    let rhs = fold(sema, ctx, e2, sink)?;
    Ok((lhs, rhs))
}

/// 6.3.5 The second operand of / and % shall not be zero, and the result shall be representable in
/// the type of the operands.
fn divisor(sema: &Sema, lhs: Value, rhs: Value) -> Result<(), Diagnosis> {
    if rhs.is_zero() {
        return Err(Diagnosis::DivisionByZero);
    }
    let fold = Fold::new(&sema.target);
    match fold.eq(rhs, Value::Int(-1)) && fold.is_min(lhs) {
        true => Err(Diagnosis::ConstantOverflow),
        false => Ok(()),
    }
}

/// 6.3.7 Bitwise shift operators
/// The right operand shall be nonnegative and less than the width in bits of the promoted left operand.
fn shift_count(sema: &Sema, lhs: Value, rhs: Value) -> Result<(), Diagnosis> {
    match Fold::new(&sema.target).shift_out_of_range(lhs, rhs) {
        true => Err(Diagnosis::ShiftCountOutOfRange),
        false => Ok(()),
    }
}

macro_rules! fold {
    ($sema:ident, $ctx:ident, $sink:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2, $sink)?;
        Ok(Fold::new(&$sema.target).$method(lhs, rhs))
    }};
}

macro_rules! fold_shift {
    ($sema:ident, $ctx:ident, $sink:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2, $sink)?;
        shift_count($sema, lhs, rhs)?;
        Ok(Fold::new(&$sema.target).$method(lhs, rhs))
    }};
}

macro_rules! fold_checked {
    ($sema:ident, $ctx:ident, $sink:ident, $expr:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2, $sink)?;
        let folded = Fold::new(&$sema.target).$method(lhs, rhs);
        Ok($sink.add_diag(folded, &$expr.span))
    }};
}

macro_rules! fold_divide {
    ($sema:ident, $ctx:ident, $sink:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2, $sink)?;
        divisor($sema, lhs, rhs)?;
        Ok(Fold::new(&$sema.target).$method(lhs, rhs))
    }};
}

macro_rules! fold_compare {
    ($sema:ident, $ctx:ident, $sink:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2, $sink)?;
        Ok(Value::from(Fold::new(&$sema.target).$method(lhs, rhs)))
    }};
}

fn fold(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode, sink: &mut Sink) -> Result<Value, Diagnosis> {
    match expr.id.resolve(ctx) {
        Expression::ConstantExpression(expr) => fold(sema, ctx, expr, sink),
        Expression::Identifier(_) => {
            let id = sema.binding(expr.id).ok_or(Diagnosis::NonConstantExpression)?;
            let symbol = id.resolve(sema);
            if symbol.kind != SymbolKind::Variant {
                return Err(Diagnosis::NonConstantExpression);
            }
            symbol.value.map(Value::Int).ok_or(Diagnosis::NonConstantExpression)
        }
        Expression::Constant(value_node) => Ok(value_node.value),
        Expression::Unary(op, operand) => match op {
            UnaryOp::Plus => fold(sema, ctx, operand, sink),
            UnaryOp::Minus => {
                let value = fold(sema, ctx, operand, sink)?;
                let folded = Fold::new(&sema.target).neg(value);
                Ok(sink.add_diag(folded, &expr.span))
            }
            UnaryOp::BitNot => {
                let value = fold(sema, ctx, operand, sink)?;
                Ok(Fold::new(&sema.target).bit_not(value))
            }
            UnaryOp::LogicalNot => Ok(fold(sema, ctx, operand, sink)?.logical_not()),
            UnaryOp::PostInc
            | UnaryOp::PostDec
            | UnaryOp::PreInc
            | UnaryOp::PreDec
            | UnaryOp::Addr
            | UnaryOp::Deref => Err(Diagnosis::NonConstantExpression),
        },
        Expression::Binary(op, e1, e2) => match op {
            BinaryOp::Add => fold_checked!(sema, ctx, sink, expr, e1, e2, add),
            BinaryOp::Sub => fold_checked!(sema, ctx, sink, expr, e1, e2, sub),
            BinaryOp::Mul => fold_checked!(sema, ctx, sink, expr, e1, e2, mul),
            BinaryOp::Div => fold_divide!(sema, ctx, sink, e1, e2, div),
            BinaryOp::Mod => fold_divide!(sema, ctx, sink, e1, e2, rem),
            BinaryOp::Left => fold_shift!(sema, ctx, sink, e1, e2, shl),
            BinaryOp::Right => fold_shift!(sema, ctx, sink, e1, e2, shr),
            BinaryOp::BitAnd => fold!(sema, ctx, sink, e1, e2, bitand),
            BinaryOp::BitOr => fold!(sema, ctx, sink, e1, e2, bitor),
            BinaryOp::BitXor => fold!(sema, ctx, sink, e1, e2, bitxor),
            BinaryOp::Greater => fold_compare!(sema, ctx, sink, e1, e2, gt),
            BinaryOp::Lower => fold_compare!(sema, ctx, sink, e1, e2, lt),
            BinaryOp::GreaterEq => fold_compare!(sema, ctx, sink, e1, e2, ge),
            BinaryOp::LowerEq => fold_compare!(sema, ctx, sink, e1, e2, le),
            BinaryOp::Eq => fold_compare!(sema, ctx, sink, e1, e2, eq),
            BinaryOp::Neq => fold_compare!(sema, ctx, sink, e1, e2, ne),
            BinaryOp::LogicalAnd => Ok(Value::from(
                fold(sema, ctx, e1, sink)?.is_true() && fold(sema, ctx, e2, sink)?.is_true(),
            )),
            BinaryOp::LogicalOr => Ok(Value::from(
                fold(sema, ctx, e1, sink)?.is_true() || fold(sema, ctx, e2, sink)?.is_true(),
            )),
        },
        Expression::Ternary(condition, e1, e2) => {
            if fold(sema, ctx, condition, sink)?.is_true() {
                fold(sema, ctx, e1, sink)
            } else {
                fold(sema, ctx, e2, sink)
            }
        }
        Expression::SizeofExpr(_) => Err(Diagnosis::InvalidSizeof),
        Expression::SizeofType(ty_node) => {
            let qualif = declaration::base_type(sema, ctx, &ty_node.specifiers, &expr.span)
                .ok_or(Diagnosis::NonConstantExpression)?;
            let (qualif, _) = declaration::declared_type(sema, ctx, Some(qualif), &ty_node.declarator)
                .ok_or(Diagnosis::NonConstantExpression)?;
            match layout::of(sema, qualif.id) {
                Some(layout) => sema
                    .target
                    .cast(&sema.target.size_t, Value::UnsignedLong(layout.size.into()))
                    .ok_or(Diagnosis::InvalidSizeof),
                None => Err(Diagnosis::InvalidSizeof),
            }
        }
        Expression::Cast(ty_node, operand) => {
            // 6.3.4's scalar-target constraint is already checked (and, if violated, reported)
            // by the type pass, which fully covers Cast; don't re-derive and re-diagnose it here.
            if sema.expr_poisoned(expr.id) {
                return Err(Diagnosis::Poisoned);
            }
            let base = declaration::base_type(sema, ctx, &ty_node.specifiers, &expr.span);
            let (qualif, _) = declaration::declared_type(sema, ctx, base, &ty_node.declarator)
                .ok_or(Diagnosis::NonConstantExpression)?;
            let val = fold(sema, ctx, operand, sink)?;
            if val.is_floating() && !matches!(operand.id.resolve(ctx), Expression::Constant(_)) {
                return Err(Diagnosis::NonConstantExpression);
            }
            cast(sema, qualif, val)
        }
        Expression::StringLiteral(_)
        | Expression::Assign(_, _, _)
        | Expression::List(_)
        | Expression::ArrayAccess(_, _)
        | Expression::FunctionCall(_, _)
        | Expression::Member(_, _, _) => Err(Diagnosis::NonConstantExpression),
    }
}
