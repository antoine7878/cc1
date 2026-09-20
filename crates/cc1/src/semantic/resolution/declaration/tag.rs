use libft::Span;

use crate::ast::{EnumId, ExpressionNode, Name, StructDeclaration, Tag};
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    Diag, Diagnostic, DiagnosticSink, Member, QualifiedType, Resolver, Symbol, SymbolKind, TagDefId, constraints,
};

pub fn struct_or_union_tag(
    resolver: &mut Resolver,
    kind: Tag,
    name: Option<Name>,
    declarations: &[StructDeclaration],
    span: &Span,
) -> TagDefId {
    let is_definition = !declarations.is_empty();
    let tag = resolver.declare_tag(kind, name, is_definition, span);
    if !is_definition {
        return tag;
    }

    let mut members: Vec<Member> = Vec::new();
    let mut has_rejected_member = false;
    for declaration in declarations {
        if declaration.struct_declarators.is_empty() {
            resolver.add_diag(Diag::err((), Diagnostic::EmptyDeclaration), &declaration.span);
        }
        let qual = base_type(resolver, &declaration.specifiers, &declaration.span);
        for declarator in &declaration.struct_declarators {
            let decl = &declarator.declarator;
            let diag_count_before = resolver.sema.diagnostics.len();
            let Some((ty, node)) = declared_type(resolver, qual, decl) else { continue };
            let already_diagnosed = resolver.sema.diagnostics.len() != diag_count_before;
            let name = node.name();
            let bit_width = declarator.bit_width.as_ref().and_then(|e| {
                let value = resolver.eval_constant(e);
                let sema = &*resolver.sema;
                let checked = constraints::types::check_bit_width(ty.id.resolve_with(sema), value, name);
                checked.collect(resolver, &e.span)
            });
            let is_member_object = ty.is_object(resolver.sema);
            if !already_diagnosed {
                constraints::types::check_member_type(is_member_object, ty).collect(resolver, &decl.span);
            }
            if already_diagnosed || !is_member_object {
                has_rejected_member = true;
                continue;
            }
            match (name, bit_width) {
                (Some(name), _) => {
                    if members
                        .iter()
                        .any(|m| m.symbol.is_some_and(|id| id.resolve_with(resolver.sema).name.id == name.id))
                    {
                        resolver.add_diag(
                            Diag::err((), Diagnostic::DuplicateDeclaration(SymbolKind::Member, name)),
                            &decl.span,
                        );
                        continue;
                    }
                    let memb = Symbol::member(name, ty, bit_width);
                    let id = resolver.sema.symbols.alloc(memb);
                    members.push(Member::symbol(id, bit_width))
                }
                (None, Some(i)) => members.push(Member::bitfield(i)),
                _ => (),
            }
        }
    }
    if !has_rejected_member && (members.is_empty() || members.iter().all(|m| m.symbol.is_none())) {
        resolver.add_diag(Diag::err((), Diagnostic::TagWithoutMember(kind.symbol_kind())), span);
    }
    resolver.sema.tags.complete(tag, members);
    tag
}

pub fn enum_tag(resolver: &mut Resolver, id: EnumId) -> Option<TagDefId> {
    let enum_node = id.resolve();
    let is_definition = !enum_node.enumerators.is_empty();
    let tag = resolver.declare_tag(Tag::Enum, enum_node.name, is_definition, &enum_node.span);
    if !is_definition {
        let is_complete = tag.resolve_with(resolver.sema).is_complete;
        constraints::types::check_enum_reference(is_complete, enum_node.name).collect(resolver, &enum_node.span);
        return Some(tag);
    }

    let mut members = Vec::new();
    let mut value: i64 = 0;

    for enumerator_id in &enum_node.enumerators {
        let enumerator = enumerator_id.resolve();
        if let Some(expr) = &enumerator.value
            && let Some(v) = enumerator_value(resolver, expr)
        {
            value = v;
        }
        if value < i32::MIN as i64 || value > i32::MAX as i64 {
            resolver.add_diag(Diag::err((), Diagnostic::EnumeratorBadValue), &enumerator.span);
            value = 0
        }
        let ty = QualifiedType::plain(resolver.sema.builtins.int);
        members.push(Member::symbol(
            resolver.declare(Symbol::enumerator(enumerator.name, ty, value as i32), &enumerator.span),
            None,
        ));
        value += 1;
    }
    resolver.sema.tags.complete(tag, members);
    Some(tag)
}

fn enumerator_value(resolver: &mut Resolver, expr: &ExpressionNode) -> Option<i64> {
    let value = resolver.eval_constant(expr)?;
    value.get_integer_value().map_or_else(
        || resolver.add_diag(Diag::err(None, Diagnostic::NonIntegerConstantExpression), &expr.span),
        |v| Some(v as i64),
    )
}
