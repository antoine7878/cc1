use crate::ast::visit::Visitor;
use crate::ast::{Expression, ExpressionNode, Fold, Tag, Value};
use crate::parser::Context;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, QualifiedType, ResolvedType, Sema, SymbolKind, declaration, layout,
};

pub fn eval_constant(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if let Some(&cached) = sema.constants.get(&expr.id) {
        return cached;
    }
    sema.visit_expression(ctx, expr);
    let value = match eval(sema, ctx, expr) {
        Ok(value) => Some(value),
        Err(diagnosis) => sema.add_diag(Diag::none_diag(diagnosis), &expr.span),
    };
    sema.constants.insert(expr.id, value);
    value
}

fn cast(sema: &mut Sema, qualif: QualifiedType, val: Value) -> Result<Value, Diagnosis> {
    let ty = sema.types.get(qualif.ty);
    if let ResolvedType::Tag(id) = ty {
        return match sema.tags.get(*id).kind {
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

/// 6.4 Constant expressions are folded for the target the unit is compiled for.
fn operands(
    sema: &mut Sema,
    ctx: &Context,
    e1: &ExpressionNode,
    e2: &ExpressionNode,
) -> Result<(Value, Value), Diagnosis> {
    let lhs = eval(sema, ctx, e1)?;
    let rhs = eval(sema, ctx, e2)?;
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
    ($sema:ident, $ctx:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2)?;
        Ok(Fold::new(&$sema.target).$method(lhs, rhs))
    }};
}

macro_rules! fold_shift {
    ($sema:ident, $ctx:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2)?;
        shift_count($sema, lhs, rhs)?;
        Ok(Fold::new(&$sema.target).$method(lhs, rhs))
    }};
}

macro_rules! fold_checked {
    ($sema:ident, $ctx:ident, $expr:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2)?;
        let folded = Fold::new(&$sema.target).$method(lhs, rhs);
        Ok($sema.add_diag(folded, &$expr.span))
    }};
}

macro_rules! fold_divide {
    ($sema:ident, $ctx:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2)?;
        divisor($sema, lhs, rhs)?;
        Ok(Fold::new(&$sema.target).$method(lhs, rhs))
    }};
}

macro_rules! fold_compare {
    ($sema:ident, $ctx:ident, $e1:ident, $e2:ident, $method:ident) => {{
        let (lhs, rhs) = operands($sema, $ctx, $e1, $e2)?;
        Ok(Value::from(Fold::new(&$sema.target).$method(lhs, rhs)))
    }};
}

pub fn eval(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Result<Value, Diagnosis> {
    match expr.id.resolve(ctx) {
        Expression::ConstantExpression(expr) => eval(sema, ctx, expr),
        Expression::Identifier(_) => {
            let id = sema
                .bindings
                .get(&expr.id)
                .copied()
                .flatten()
                .ok_or(Diagnosis::NonConstantExpression)?;
            let symbol = sema.symbols.get(id);
            if symbol.kind != SymbolKind::Variant {
                return Err(Diagnosis::NonConstantExpression);
            }
            symbol.value.map(Value::Int).ok_or(Diagnosis::NonConstantExpression)
        }
        Expression::Constant(value_node) => Ok(value_node.value),
        Expression::Plus(expr) => eval(sema, ctx, expr),
        Expression::Minus(operand) => {
            let value = eval(sema, ctx, operand)?;
            let folded = Fold::new(&sema.target).neg(value);
            Ok(sema.add_diag(folded, &expr.span))
        }
        Expression::BitNot(expr) => {
            let value = eval(sema, ctx, expr)?;
            Ok(Fold::new(&sema.target).bit_not(value))
        }
        Expression::LogicalNot(expr) => Ok(eval(sema, ctx, expr)?.logical_not()),
        Expression::Add(e1, e2) => fold_checked!(sema, ctx, expr, e1, e2, add),
        Expression::Sub(e1, e2) => fold_checked!(sema, ctx, expr, e1, e2, sub),
        Expression::Mul(e1, e2) => fold_checked!(sema, ctx, expr, e1, e2, mul),
        Expression::Div(e1, e2) => fold_divide!(sema, ctx, e1, e2, div),
        Expression::Mod(e1, e2) => fold_divide!(sema, ctx, e1, e2, rem),
        Expression::Left(e1, e2) => fold_shift!(sema, ctx, e1, e2, shl),
        Expression::Right(e1, e2) => fold_shift!(sema, ctx, e1, e2, shr),
        Expression::BitAnd(e1, e2) => fold!(sema, ctx, e1, e2, bitand),
        Expression::BitOr(e1, e2) => fold!(sema, ctx, e1, e2, bitor),
        Expression::BitXor(e1, e2) => fold!(sema, ctx, e1, e2, bitxor),
        Expression::Greater(e1, e2) => fold_compare!(sema, ctx, e1, e2, gt),
        Expression::Lower(e1, e2) => fold_compare!(sema, ctx, e1, e2, lt),
        Expression::GreaterEq(e1, e2) => fold_compare!(sema, ctx, e1, e2, ge),
        Expression::LowerEq(e1, e2) => fold_compare!(sema, ctx, e1, e2, le),
        Expression::Eq(e1, e2) => fold_compare!(sema, ctx, e1, e2, eq),
        Expression::Neq(e1, e2) => fold_compare!(sema, ctx, e1, e2, ne),
        Expression::LogicalAnd(e1, e2) => Ok(Value::from(
            eval(sema, ctx, e1)?.is_true() && eval(sema, ctx, e2)?.is_true(),
        )),
        Expression::LogicalOr(e1, e2) => Ok(Value::from(
            eval(sema, ctx, e1)?.is_true() || eval(sema, ctx, e2)?.is_true(),
        )),
        Expression::Ternary(condition, e1, e2) => {
            if eval(sema, ctx, condition)?.is_true() {
                eval(sema, ctx, e1)
            } else {
                eval(sema, ctx, e2)
            }
        }
        Expression::SizeofExpr(_) => Err(Diagnosis::InvalidSizeof),
        Expression::SizeofType(ty_node) => {
            let qualif = declaration::base_type(sema, ctx, &ty_node.specifiers, &expr.span)
                .ok_or(Diagnosis::NonConstantExpression)?;
            let (qualif, _) = declaration::declared_type(sema, ctx, Some(qualif), &ty_node.declarator)
                .ok_or(Diagnosis::NonConstantExpression)?;
            match layout::of(sema, qualif.ty) {
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
            let val = eval(sema, ctx, operand)?;
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
