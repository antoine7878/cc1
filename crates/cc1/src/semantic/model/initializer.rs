use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::visit::Visitor;
use crate::ast::{ConstValue, Expression, ExpressionNode, InitializerNode, StringConstId, Tag};
use crate::semantic::resolution::expression;
use crate::semantic::{
    AssignmentContext, Diag, DiagCollector, Diagnosis, Duration, Place, QualifiedType, ResolvedType, Sema,
    SymbolResolver, TagDefId, address, ice,
};
use crate::{ast, define_arena};

define_arena!(Initializer, InitializerArena, InitializerId);

#[derive(Clone, Debug)]
pub enum Initializer {
    Zero,
    Value(ConstValue),
    Address(Place),
    String(StringConstId),
    List(Vec<Initializer>),
    Expr(ExpressionNode),
}

impl Initializer {
    pub fn len(&self) -> Option<usize> {
        match self {
            Initializer::List(values) => Some(values.len()),
            Initializer::String(id) => Some(id.resolve().units.len() + 1),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> Option<bool> {
        unimplemented!()
    }
}

type Cursor<'a> = Peekable<Iter<'a, InitializerNode>>;
pub fn resolve(
    resolver: &mut SymbolResolver,
    ty: QualifiedType,
    node: &InitializerNode,
    duration: Duration,
) -> Initializer {
    let constant = duration == Duration::Static;
    match &node.init {
        ast::Initializer::Single(e) => single(resolver, ty, e, constant),
        ast::Initializer::List(items) => braced(resolver, ty, items, constant),
    }
}

fn single(resolver: &mut SymbolResolver, ty: QualifiedType, e: &ExpressionNode, constant: bool) -> Initializer {
    if let Some(init) = string(resolver, ty, e) {
        return init;
    }
    resolver.visit_expression(e);
    if let Err(inner) = expression::init(resolver.sema, ty, e, AssignmentContext::Initialization) {
        resolver.add_diag(Diag::err((), inner), &e.span);
        return Initializer::Zero;
    }
    if !constant {
        return Initializer::Expr(e.clone());
    }
    if let Some(value) = ice::try_fold(resolver.sema, e) {
        return Initializer::Value(value);
    }
    if let Some(at) = address::fold(resolver.sema, e) {
        return Initializer::Address(at);
    }
    resolver.add_diag(Diag::err((), Diagnosis::NonConstantInitializer), &e.span);
    Initializer::Zero
}

fn braced(resolver: &mut SymbolResolver, ty: QualifiedType, items: &[InitializerNode], constant: bool) -> Initializer {
    let mut cursor = items.iter().peekable();
    let value = match string_at(resolver, ty, &mut cursor) {
        Some(init) => init,
        None => match is_aggregate(resolver.sema, ty) {
            true => fill(resolver, ty, &mut cursor, true),
            false => walk(resolver, ty, &mut cursor, constant),
        },
    };
    excess(resolver, &mut cursor);
    value
}

fn string_at(resolver: &mut SymbolResolver, ty: QualifiedType, cursor: &mut Cursor) -> Option<Initializer> {
    let ast::Initializer::Single(e) = &cursor.peek()?.init else { return None };
    let init = string(resolver, ty, e)?;
    cursor.next();
    Some(init)
}

fn fill(resolver: &mut SymbolResolver, ty: QualifiedType, cursor: &mut Cursor, constant: bool) -> Initializer {
    let mut values = Vec::new();
    match ty.id.resolve_with(resolver.sema).clone() {
        ResolvedType::Array { elem, len } => {
            while len.is_none_or(|len| values.len() < len) && cursor.peek().is_some() {
                values.push(walk(resolver, elem, cursor, constant));
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
                values.push(walk(resolver, *member, cursor, constant));
            }
            values.resize(members.len(), Initializer::Zero);
        }
        _ => unreachable!("fill on a non-aggregate type"),
    }
    Initializer::List(values)
}

fn walk(resolver: &mut SymbolResolver, ty: QualifiedType, cursor: &mut Cursor, constant: bool) -> Initializer {
    if let Some(init) = string_at(resolver, ty, cursor) {
        return init;
    }
    if !is_aggregate(resolver.sema, ty) {
        return match cursor.next() {
            None => Initializer::Zero,
            Some(node) => match &node.init {
                ast::Initializer::Single(e) => single(resolver, ty, e, constant),
                ast::Initializer::List(items) => braced(resolver, ty, items, constant),
            },
        };
    }
    match cursor.peek() {
        Some(node) => match &node.init {
            ast::Initializer::List(items) => {
                cursor.next();
                braced(resolver, ty, items, true)
            }
            ast::Initializer::Single(_) => fill(resolver, ty, cursor, true),
        },
        None => fill(resolver, ty, cursor, true),
    }
}

fn string(resolver: &mut SymbolResolver, ty: QualifiedType, e: &ExpressionNode) -> Option<Initializer> {
    let &ResolvedType::Array { elem, len } = ty.id.resolve_with(resolver.sema) else {
        return None;
    };
    if !elem.id.resolve_with(resolver.sema).is_char() {
        return None;
    }
    let Expression::StringLiteral(literal) = e.id.resolve() else {
        return None;
    };
    if literal.is_wide() {
        return None;
    }
    let id = literal.id;
    resolver.visit_expression(e);
    if len.is_some_and(|len| len < literal.len()) {
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
    match ty.id.resolve_with(sema) {
        ResolvedType::Array { .. } => true,
        ResolvedType::Tag(id) => matches!(sema.tags.get(*id).kind, Tag::Struct | Tag::Union),
        _ => false,
    }
}

fn member_types(sema: &Sema, id: TagDefId) -> Vec<QualifiedType> {
    let tag = id.resolve_with(sema);
    let named = tag.members.iter().filter_map(|member| member.sym).map(|sym| sym.resolve_with(sema).ty);
    match tag.kind {
        Tag::Union => named.take(1).collect(),
        _ => named.collect(),
    }
}
