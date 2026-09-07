use crate::arena::ResolveWith;
use crate::ast::{EnumId, ExpressionNode, Name, StructDeclaration, Tag};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, Member, QualifiedType, Sema, Symbol, SymbolKind, TagDefId, constrain, ice,
};

pub fn struct_or_union_tag(
    sema: &mut Sema,
    ctx: &Context,
    kind: Tag,
    name: Option<Name>,
    fields: &[StructDeclaration],
    span: &Span,
) -> TagDefId {
    let is_definition = !fields.is_empty();
    let tag = sema.declare_tag(kind, name, is_definition, span);
    if !is_definition {
        return tag;
    }

    let mut members: Vec<Member> = Vec::new();
    let mut has_rejected_member = false;
    for field in fields {
        if field.struct_declarators.is_empty() {
            sema.add_diag(Diag::err((), Diagnosis::EmptyDeclaration), &field.span);
        }
        let qual = base_type(sema, ctx, &field.specifiers, &field.span);
        for declarator in &field.struct_declarators {
            let decl = &declarator.declarator;
            let diag_count_before = sema.diagnosis.len();
            let Some((ty, node)) = declared_type(sema, ctx, qual, decl) else { continue };
            let already_diagnosed = sema.diagnosis.len() != diag_count_before;
            let bit_width = declarator.bit_width.as_ref().and_then(|e| {
                let value = ice::eval_constant(sema, ctx, e);
                constrain::declaration::check_bit_width(ty.id.resolve(sema), value).collect(sema, span)
            });
            let is_member_object = ty.is_object(sema);
            if !already_diagnosed {
                constrain::declaration::check_member_type(is_member_object, ty).collect(sema, &decl.span);
            }
            if already_diagnosed || !is_member_object {
                has_rejected_member = true;
                continue;
            }
            match (node.ident(ctx), bit_width) {
                (Some(name), _) => {
                    if members
                        .iter()
                        .any(|m| m.sym.is_some_and(|id| id.resolve(sema).name.id == name.id))
                    {
                        sema.add_diag(
                            Diag::err((), Diagnosis::DuplicateDeclaration(SymbolKind::Member, name)),
                            &decl.span,
                        );
                        continue;
                    }
                    let memb = Symbol::member(name, ty, bit_width);
                    let id = sema.symbols.alloc(memb);
                    members.push(Member::symbol(id, bit_width))
                }
                (None, Some(i)) => members.push(Member::bitfield(i)),
                _ => (),
            }
        }
    }
    if !has_rejected_member && (members.is_empty() || members.iter().all(|m| m.sym.is_none())) {
        sema.add_diag(Diag::err((), Diagnosis::TagWithoutMember(kind.symbol_kind())), span);
    }
    sema.tags.complete(tag, members);
    tag
}

pub fn enum_tag(sema: &mut Sema, ctx: &Context, id: EnumId) -> Option<TagDefId> {
    let enum_node = id.resolve(ctx);
    let is_definition = !enum_node.variants.is_empty();
    let tag = sema.declare_tag(Tag::Enum, enum_node.name, is_definition, &enum_node.span);
    if !is_definition {
        return Some(tag);
    }

    let mut members = Vec::new();
    let mut value: i64 = 0;

    for variant_id in &enum_node.variants {
        let variant = variant_id.resolve(ctx);
        if let Some(expr) = &variant.value
            && let Some(v) = variant_value(sema, ctx, expr)
        {
            value = v;
        }
        if value < i32::MIN as i64 || value > i32::MAX as i64 {
            sema.add_diag(Diag::err((), Diagnosis::VariantBadValue), &variant.span);
            value = 0
        }
        let ty = QualifiedType::plain(sema.builtins.int);
        members.push(Member::symbol(
            sema.declare(Symbol::variant(variant.name, ty, value as i32), &variant.span),
            None,
        ));
        value += 1;
    }
    sema.tags.complete(tag, members);
    Some(tag)
}

fn variant_value(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<i64> {
    let value = ice::eval_constant(sema, ctx, expr)?;
    value.get_integer_value().map_or_else(
        || sema.add_diag(Diag::err(None, Diagnosis::NonIntegerConstantExpression), &expr.span),
        |v| Some(v as i64),
    )
}
