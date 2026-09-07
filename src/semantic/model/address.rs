use crate::arena::ResolveWith;
use crate::ast::{BinaryOp, Expression, ExpressionNode, MemberOp, StringConstId, UnaryOp};
use crate::context::Context;
use crate::semantic::{Duration, ResolvedType, Sema, SymbolId, SymbolKind, ice, layout};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressBase {
    Symbol(SymbolId),
    String(StringConstId),
    Absolute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Place {
    pub base: AddressBase,
    pub offset: i64,
}

impl Place {
    fn at(base: AddressBase, offset: i64) -> Self {
        Self { base, offset }
    }

    fn shift(self, delta: i64) -> Option<Self> {
        Some(Self::at(self.base, self.offset.checked_add(delta)?))
    }
}

pub fn fold(sema: &mut Sema, ctx: &Context, e: &ExpressionNode) -> Option<Place> {
    if decays(sema, e) {
        return place(sema, ctx, e);
    }
    match e.id.resolve(ctx) {
        Expression::ConstantExpression(inner) => fold(sema, ctx, inner),
        Expression::StringLiteral(literal) => Some(Place::at(AddressBase::String(literal.id), 0)),
        Expression::Unary(UnaryOp::Addr, inner) => place(sema, ctx, inner),
        Expression::Cast(_, inner) => cast(sema, ctx, e, inner),
        Expression::Binary(BinaryOp::Add, e1, e2) => match additive(sema, ctx, e1, e2, 1) {
            Some(at) => Some(at),
            None => additive(sema, ctx, e2, e1, 1),
        },
        Expression::Binary(BinaryOp::Sub, e1, e2) => additive(sema, ctx, e1, e2, -1),
        _ => None,
    }
}

fn place(sema: &mut Sema, ctx: &Context, e: &ExpressionNode) -> Option<Place> {
    match e.id.resolve(ctx) {
        Expression::ConstantExpression(inner) => place(sema, ctx, inner),
        Expression::Identifier(_) => object(sema, e),
        Expression::StringLiteral(literal) => Some(Place::at(AddressBase::String(literal.id), 0)),
        Expression::Unary(UnaryOp::Deref, inner) => fold(sema, ctx, inner),
        Expression::ArraySubscripting(base, index) => match additive(sema, ctx, base, index, 1) {
            Some(at) => Some(at),
            None => additive(sema, ctx, index, base, 1),
        },
        Expression::Member(op, base, _) => member(sema, ctx, e, base, *op),
        _ => None,
    }
}

fn object(sema: &Sema, e: &ExpressionNode) -> Option<Place> {
    let sym_id = sema.expr_bindings.get(e.id).copied()?;
    let sym = sym_id.resolve(sema);
    let addressable = sym.duration == Duration::Static || sym.kind == SymbolKind::Function;
    addressable.then(|| Place::at(AddressBase::Symbol(sym_id), 0))
}

fn decays(sema: &Sema, e: &ExpressionNode) -> bool {
    let Some(re) = sema.expr_types.get(e.id) else { return false };
    matches!(
        re.ty.id.resolve(sema),
        ResolvedType::Array { .. } | ResolvedType::Function { .. }
    )
}

fn member(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, base: &ExpressionNode, op: MemberOp) -> Option<Place> {
    let reference = sema.member_refs.get(node.id).copied()?;
    layout::of_tag(sema, reference.tag)?;
    let offset = reference.member(sema).offset;
    let at = match op {
        MemberOp::Dot => place(sema, ctx, base)?,
        MemberOp::Arrow => fold(sema, ctx, base)?,
    };
    at.shift(i64::from(offset))
}

fn additive(sema: &mut Sema, ctx: &Context, ptr: &ExpressionNode, index: &ExpressionNode, sign: i64) -> Option<Place> {
    let size = pointee_size(sema, ptr)?;
    let count = integer(sema, ctx, index)?;
    let at = fold(sema, ctx, ptr)?;
    at.shift(count.checked_mul(size)?.checked_mul(sign)?)
}

fn cast(sema: &mut Sema, ctx: &Context, node: &ExpressionNode, inner: &ExpressionNode) -> Option<Place> {
    if let Some(at) = fold(sema, ctx, inner) {
        return Some(at);
    }
    let ty = sema.expr_types.get(node.id)?.casted_ty();
    if !ty.is_pointer(sema) {
        return None;
    }
    Some(Place::at(AddressBase::Absolute, integer(sema, ctx, inner)?))
}

fn pointee_size(sema: &mut Sema, e: &ExpressionNode) -> Option<i64> {
    let ty = sema.expr_types.get(e.id)?.casted_ty();
    let &ResolvedType::Pointer(inner) = ty.id.resolve(sema) else { return None };
    layout::of(sema, inner.id).map(|layout| i64::from(layout.size))
}

fn integer(sema: &mut Sema, ctx: &Context, e: &ExpressionNode) -> Option<i64> {
    let value = ice::try_fold(sema, ctx, e)?;
    value.get_integer_value()?;
    Some(value.to_i64())
}
