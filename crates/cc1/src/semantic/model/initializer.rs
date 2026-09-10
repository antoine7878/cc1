use std::iter::Peekable;
use std::slice::Iter;

use crate::arena::ResolveWith;
use crate::ast;
use crate::ast::visit::Visitor;
use crate::ast::{Expression, ExpressionId, ExpressionNode, InitializerNode, StringConstId, Tag, Value};
use crate::context::Context;
use crate::define_arena;
use crate::semantic::resolution::expression;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, Duration, Place, QualifiedType, ResolvedType, Sema,
    SymbolResolver, TagDefId, address, ice,
};

define_arena!(Initializer, InitializerArena, InitializerId, Sema, crate::semantic::sema(), inits);

#[derive(Clone, Debug)]
pub enum Initializer {
    Zero,
    Value(Value),
    Address(Place),
    String(StringConstId),
    List(Vec<Initializer>),
    Expr(ExpressionId),
}

impl Initializer {
    pub fn len(&self, ctx: &Context) -> Option<usize> {
        match self {
            Initializer::List(values) => Some(values.len()),
            Initializer::String(id) => Some(id.resolve().units.len() + 1),
            _ => None,
        }
    }
}

type Cursor<'a> = Peekable<Iter<'a, InitializerNode>>;
pub fn resolve(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    ty: QualifiedType,
    node: &InitializerNode,
    duration: Duration,
) -> Initializer {
    let constant = duration == Duration::Static;
    match &node.init {
        ast::Initializer::Single(e) => single(resolver, ctx, ty, e, constant),
        ast::Initializer::List(items) => braced(resolver, ctx, ty, items, constant),
    }
}

fn single(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    ty: QualifiedType,
    e: &ExpressionNode,
    constant: bool,
) -> Initializer {
    if let Some(init) = string(resolver, ctx, ty, e) {
        return init;
    }
    resolver.visit_expression(ctx, e);
    if let Err(inner) = expression::init(resolver.sema, ctx, ty, e, AssignmentContext::Initialization) {
        resolver.add_diag(Diag::err((), inner), &e.span);
        return Initializer::Zero;
    }
    if !constant {
        return Initializer::Expr(e.id);
    }
    if let Some(value) = ice::try_fold(resolver.sema, ctx, e) {
        return Initializer::Value(value);
    }
    if let Some(at) = address::fold(resolver.sema, ctx, e) {
        return Initializer::Address(at);
    }
    resolver.add_diag(Diag::err((), Diagnosis::NonConstantInitializer), &e.span);
    Initializer::Zero
}

fn braced(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    ty: QualifiedType,
    items: &[InitializerNode],
    constant: bool,
) -> Initializer {
    let mut cursor = items.iter().peekable();
    let value = match string_at(resolver, ctx, ty, &mut cursor) {
        Some(init) => init,
        None => match is_aggregate(resolver.sema, ty) {
            true => fill(resolver, ctx, ty, &mut cursor, true),
            false => walk(resolver, ctx, ty, &mut cursor, constant),
        },
    };
    excess(resolver, &mut cursor);
    value
}

fn string_at(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    ty: QualifiedType,
    cursor: &mut Cursor,
) -> Option<Initializer> {
    let ast::Initializer::Single(e) = &cursor.peek()?.init else { return None };
    let init = string(resolver, ctx, ty, e)?;
    cursor.next();
    Some(init)
}

fn fill(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    ty: QualifiedType,
    cursor: &mut Cursor,
    constant: bool,
) -> Initializer {
    let mut values = Vec::new();
    match ty.id.resolve_in(resolver.sema).clone() {
        ResolvedType::Array { elem, len } => {
            while len.is_none_or(|len| values.len() < len) && cursor.peek().is_some() {
                values.push(walk(resolver, ctx, elem, cursor, constant));
            }
            if let Some(len) = len {
                values.resize(len, Initializer::Zero);
            }
        }
        ResolvedType::Tag(id) => {
            let members = member_types(resolver.sema, id);
            for member in &members {
                if cursor.peek().is_none() {
                    break;
                }
                values.push(walk(resolver, ctx, *member, cursor, constant));
            }
            values.resize(members.len(), Initializer::Zero);
        }
        _ => unreachable!("fill on a non-aggregate type"),
    }
    Initializer::List(values)
}

fn walk(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    ty: QualifiedType,
    cursor: &mut Cursor,
    constant: bool,
) -> Initializer {
    if let Some(init) = string_at(resolver, ctx, ty, cursor) {
        return init;
    }
    if !is_aggregate(resolver.sema, ty) {
        return match cursor.next() {
            None => Initializer::Zero,
            Some(node) => match &node.init {
                ast::Initializer::Single(e) => single(resolver, ctx, ty, e, constant),
                ast::Initializer::List(items) => braced(resolver, ctx, ty, items, constant),
            },
        };
    }
    match cursor.peek() {
        Some(node) => match &node.init {
            ast::Initializer::List(items) => {
                cursor.next();
                braced(resolver, ctx, ty, items, true)
            }
            ast::Initializer::Single(_) => fill(resolver, ctx, ty, cursor, true),
        },
        None => fill(resolver, ctx, ty, cursor, true),
    }
}

fn string(resolver: &mut SymbolResolver, ctx: &Context, ty: QualifiedType, e: &ExpressionNode) -> Option<Initializer> {
    let &ResolvedType::Array { elem, len } = ty.id.resolve_in(resolver.sema) else {
        return None;
    };
    if !elem.id.resolve_in(resolver.sema).is_char() {
        return None;
    }
    let Expression::StringLiteral(literal) = e.id.resolve() else {
        return None;
    };
    if literal.is_wide(ctx) {
        return None;
    }
    let id = literal.id;
    resolver.visit_expression(ctx, e);
    if len.is_some_and(|len| len < literal.len(ctx)) {
        resolver.add_diag(Diag::err((), Diagnosis::ArrayInitTooLong), &e.span);
    }
    Some(Initializer::String(id))
}

fn excess(resolver: &mut SymbolResolver, cursor: &mut Cursor) {
    if let Some(node) = cursor.next() {
        resolver.add_diag(Diag::err((), Diagnosis::ArrayInitTooLong), &node.span);
    }
}

fn is_aggregate(sema: &Sema, ty: QualifiedType) -> bool {
    match ty.id.resolve_in(sema) {
        ResolvedType::Array { .. } => true,
        ResolvedType::Tag(id) => matches!(sema.tags.get(*id).kind, Tag::Struct | Tag::Union),
        _ => false,
    }
}

fn member_types(sema: &Sema, id: TagDefId) -> Vec<QualifiedType> {
    let tag = id.resolve_in(sema);
    let named = tag
        .members
        .iter()
        .filter_map(|member| member.sym)
        .filter_map(|sym| sym.resolve_in(sema).ty);
    match tag.kind {
        Tag::Union => named.take(1).collect(),
        _ => named.collect(),
    }
}
