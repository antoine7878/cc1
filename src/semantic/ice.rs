use crate::ast::visit::Visitor;
use crate::ast::{Expression, ExpressionNode, Tag, Value};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::sema::Sema;
use crate::semantic::{Diag, DiagCollector, QualifiedType, ResolvedType, SymbolKind, layout, ty};

pub fn eval_constant(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
    if let Some(&cached) = sema.const_values.get(&expr.id) {
        return cached;
    }
    sema.visit_expression(ctx, expr);
    let value = match eval(sema, ctx, expr) {
        Ok(value) => Some(value),
        Err(diagnosis) => sema.add_diag(Diag::none_diag(diagnosis), &expr.span),
    };
    sema.const_values.insert(expr.id, value);
    value
}

fn cast(sema: &mut Sema, qualif: QualifiedType, val: Value, span: &Span) -> Result<Value, Diagnosis> {
    let ty = *sema.types.get(qualif.ty);
    if let ResolvedType::Tag(id) = ty {
        return match sema.tags.get(id).kind {
            Tag::Enum => sema
                .target
                .cast(&ResolvedType::Int, val)
                .ok_or(Diagnosis::NonIntegerConstantExpression),
            _ => Err(Diagnosis::CastToNonScalar),
        };
    }
    if let Some(casted) = sema.target.cast(&ty, val) {
        return Ok(casted);
    }
    let _ = span;
    match ty {
        ResolvedType::Array { .. } | ResolvedType::Void => Err(Diagnosis::CastToNonScalar),
        _ => Err(Diagnosis::NonIntegerConstantExpression),
    }
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
        Expression::Minus(expr) => Ok(-eval(sema, ctx, expr)?),
        Expression::BitNot(expr) => Ok(!eval(sema, ctx, expr)?),
        Expression::LogicalNot(expr) => Ok(eval(sema, ctx, expr)?.logical_not()),
        Expression::Add(e1, e2) => Ok(eval(sema, ctx, e1)? + eval(sema, ctx, e2)?),
        Expression::Sub(e1, e2) => Ok(eval(sema, ctx, e1)? - eval(sema, ctx, e2)?),
        Expression::Mul(e1, e2) => Ok(eval(sema, ctx, e1)? * eval(sema, ctx, e2)?),
        Expression::Div(e1, e2) => Ok(eval(sema, ctx, e1)? / eval(sema, ctx, e2)?),
        Expression::Mod(e1, e2) => Ok(eval(sema, ctx, e1)? % eval(sema, ctx, e2)?),
        Expression::Left(e1, e2) => Ok(eval(sema, ctx, e1)? << eval(sema, ctx, e2)?),
        Expression::Right(e1, e2) => Ok(eval(sema, ctx, e1)? >> eval(sema, ctx, e2)?),
        Expression::BitAnd(e1, e2) => Ok(eval(sema, ctx, e1)? & eval(sema, ctx, e2)?),
        Expression::BitOr(e1, e2) => Ok(eval(sema, ctx, e1)? | eval(sema, ctx, e2)?),
        Expression::BitXor(e1, e2) => Ok(eval(sema, ctx, e1)? ^ eval(sema, ctx, e2)?),
        Expression::Greater(e1, e2) => Ok(Value::from(eval(sema, ctx, e1)? > eval(sema, ctx, e2)?)),
        Expression::Lower(e1, e2) => Ok(Value::from(eval(sema, ctx, e1)? < eval(sema, ctx, e2)?)),
        Expression::GreaterEq(e1, e2) => Ok(Value::from(eval(sema, ctx, e1)? >= eval(sema, ctx, e2)?)),
        Expression::LowerEq(e1, e2) => Ok(Value::from(eval(sema, ctx, e1)? <= eval(sema, ctx, e2)?)),
        Expression::Eq(e1, e2) => Ok(Value::from(eval(sema, ctx, e1)? == eval(sema, ctx, e2)?)),
        Expression::Neq(e1, e2) => Ok(Value::from(eval(sema, ctx, e1)? != eval(sema, ctx, e2)?)),
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
            let qualif =
                ty::resolve_type(sema, ctx, &ty_node.specifiers, &expr.span).ok_or(Diagnosis::NonConstantExpression)?;
            let (qualif, _) = ty::make_qualified_type(sema, ctx, Some(qualif), &ty_node.declarator)
                .ok_or(Diagnosis::NonConstantExpression)?;
            Ok(Value::UnsignedLong(layout::of(sema, qualif)?.size.into()))
        }
        Expression::Cast(ty_node, operand) => {
            let base = ty::resolve_type(sema, ctx, &ty_node.specifiers, &expr.span);
            let (qualif, _) = ty::make_qualified_type(sema, ctx, base, &ty_node.declarator)
                .ok_or(Diagnosis::NonConstantExpression)?;
            let val = eval(sema, ctx, operand)?;
            if val.is_floating() && !matches!(operand.id.resolve(ctx), Expression::Constant(_)) {
                return Err(Diagnosis::NonConstantExpression);
            }
            cast(sema, qualif, val, &expr.span)
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
