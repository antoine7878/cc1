use crate::arena::ResolveWith;
use crate::ast::visit::Visitor;
use crate::ast::{Expression, ExpressionNode, Fold, Tag, Value};
use crate::context::Context;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, QualifiedType, ResolvedType, Sema, SymbolKind, declaration, layout,
};

struct Sink<'a>(&'a mut Vec<DiagnosisNode>);

impl DiagCollector for Sink<'_> {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        self.0
    }
}

pub fn eval_constant(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if let Some(&cached) = sema.constants.get(&expr.id) {
        return cached;
    }
    sema.visit_expression(ctx, expr);
    let mut collected = Vec::new();
    let folded = fold(sema, ctx, expr, &mut Sink(&mut collected));
    sema.diagnosis.append(&mut collected);
    let value = match folded {
        Ok(value) => Some(value),
        Err(diagnosis) => sema.add_diag(Diag::none_diag(diagnosis), &expr.span),
    };
    sema.constants.insert(expr.id, value);
    value
}

pub fn try_fold(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if let Some(&cached) = sema.constants.get(&expr.id) {
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
            _ => Err(Diagnosis::Poisoned),
        };
    }
    if let Some(casted) = sema.target.cast(ty, val) {
        return Ok(casted);
    }
    match ty {
        ResolvedType::Array { .. } | ResolvedType::Void => Err(Diagnosis::Poisoned),
        _ => Err(Diagnosis::NonIntegerConstantExpression),
    }
}

/// 6.4 Constant expressions are folded for the target the unit is compiled for.
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
            let id = sema
                .bindings
                .get(&expr.id)
                .copied()
                .flatten()
                .ok_or(Diagnosis::NonConstantExpression)?;
            let symbol = id.resolve(sema);
            if symbol.kind != SymbolKind::Variant {
                return Err(Diagnosis::NonConstantExpression);
            }
            symbol.value.map(Value::Int).ok_or(Diagnosis::NonConstantExpression)
        }
        Expression::Constant(value_node) => Ok(value_node.value),
        Expression::Plus(expr) => fold(sema, ctx, expr, sink),
        Expression::Minus(operand) => {
            let value = fold(sema, ctx, operand, sink)?;
            let folded = Fold::new(&sema.target).neg(value);
            Ok(sink.add_diag(folded, &expr.span))
        }
        Expression::BitNot(expr) => {
            let value = fold(sema, ctx, expr, sink)?;
            Ok(Fold::new(&sema.target).bit_not(value))
        }
        Expression::LogicalNot(expr) => Ok(fold(sema, ctx, expr, sink)?.logical_not()),
        Expression::Add(e1, e2) => fold_checked!(sema, ctx, sink, expr, e1, e2, add),
        Expression::Sub(e1, e2) => fold_checked!(sema, ctx, sink, expr, e1, e2, sub),
        Expression::Mul(e1, e2) => fold_checked!(sema, ctx, sink, expr, e1, e2, mul),
        Expression::Div(e1, e2) => fold_divide!(sema, ctx, sink, e1, e2, div),
        Expression::Mod(e1, e2) => fold_divide!(sema, ctx, sink, e1, e2, rem),
        Expression::Left(e1, e2) => fold_shift!(sema, ctx, sink, e1, e2, shl),
        Expression::Right(e1, e2) => fold_shift!(sema, ctx, sink, e1, e2, shr),
        Expression::BitAnd(e1, e2) => fold!(sema, ctx, sink, e1, e2, bitand),
        Expression::BitOr(e1, e2) => fold!(sema, ctx, sink, e1, e2, bitor),
        Expression::BitXor(e1, e2) => fold!(sema, ctx, sink, e1, e2, bitxor),
        Expression::Greater(e1, e2) => fold_compare!(sema, ctx, sink, e1, e2, gt),
        Expression::Lower(e1, e2) => fold_compare!(sema, ctx, sink, e1, e2, lt),
        Expression::GreaterEq(e1, e2) => fold_compare!(sema, ctx, sink, e1, e2, ge),
        Expression::LowerEq(e1, e2) => fold_compare!(sema, ctx, sink, e1, e2, le),
        Expression::Eq(e1, e2) => fold_compare!(sema, ctx, sink, e1, e2, eq),
        Expression::Neq(e1, e2) => fold_compare!(sema, ctx, sink, e1, e2, ne),
        Expression::LogicalAnd(e1, e2) => Ok(Value::from(
            fold(sema, ctx, e1, sink)?.is_true() && fold(sema, ctx, e2, sink)?.is_true(),
        )),
        Expression::LogicalOr(e1, e2) => Ok(Value::from(
            fold(sema, ctx, e1, sink)?.is_true() || fold(sema, ctx, e2, sink)?.is_true(),
        )),
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
        | Expression::PostInc(_)
        | Expression::PostDec(_)
        | Expression::PreInc(_)
        | Expression::Deref(_)
        | Expression::Addr(_)
        | Expression::Assign(_, _)
        | Expression::MulAssign(_, _)
        | Expression::DivAssign(_, _)
        | Expression::ModAssign(_, _)
        | Expression::AddAssign(_, _)
        | Expression::SubAssign(_, _)
        | Expression::LeftAssign(_, _)
        | Expression::RightAssign(_, _)
        | Expression::AndAssign(_, _)
        | Expression::XorAssign(_, _)
        | Expression::OrAssign(_, _)
        | Expression::List(_, _)
        | Expression::ArrayAcces(_, _)
        | Expression::FunctionCall(_, _)
        | Expression::DotAcces(_, _)
        | Expression::PtrAcces(_, _)
        | Expression::PreDec(_) => Err(Diagnosis::NonConstantExpression),
    }
}
