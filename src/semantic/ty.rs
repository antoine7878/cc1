use crate::ast::{
    DeclarationSpecifier, Declarator, DeclaratorNode, EnumId, ExpressionNode, Name, StructDeclaration, Tag,
    TypeSpecifier,
};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::sema::Sema;
use crate::semantic::symbol::Symbol;
use crate::semantic::{Diag, DiagCollector, QualifiedType, SymbolKind, TagDefId, constrain, ice};

/// 6.5.2 Type specifiers
/// Each list of type specifiers shall be one of the following sets...
pub fn resolve_type(
    sema: &mut Sema,
    ctx: &Context,
    specifiers: &[DeclarationSpecifier],
    span: &Span,
) -> Option<QualifiedType> {
    let (is_const, is_volatile) = constrain::declaration::get_qualifier(specifiers).collect(sema, span);

    let types: Vec<_> = specifiers
        .iter()
        .filter_map(|s| match s {
            DeclarationSpecifier::Type(t) => Some(t),
            _ => None,
        })
        .collect();

    let id = match types.as_slice() {
        [TypeSpecifier::Struct(t)] => {
            let node = t.resolve(ctx);
            let tag = resolve_struct_or_union(sema, ctx, Tag::Struct, node.name, &node.fields, &node.span);
            sema.types.tag(tag)
        }
        [TypeSpecifier::Union(t)] => {
            let node = t.resolve(ctx);
            let tag = resolve_struct_or_union(sema, ctx, Tag::Union, node.name, &node.fields, &node.span);
            sema.types.tag(tag)
        }
        [TypeSpecifier::Enum(t)] => {
            let tag = resolve_enum(sema, ctx, *t)?;
            sema.types.tag(tag)
        }
        [TypeSpecifier::TypedefName(t)] => return sema.resolve_typedef(*t, is_const, is_volatile, span),
        s => {
            let ty = constrain::declaration::basic_type(s).collect(sema, span)?;
            sema.types.alloc(ty)
        }
    };
    Some(QualifiedType::new(id, is_const, is_volatile))
}

pub fn make_qualified_type(
    sema: &mut Sema,
    ctx: &Context,
    inner_most: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode)> {
    Some(extract_declarator(sema, ctx, decl, inner_most?))
}

fn extract_declarator(
    sema: &mut Sema,
    ctx: &Context,
    declarator: &DeclaratorNode,
    inner_most: QualifiedType,
) -> (QualifiedType, DeclaratorNode) {
    match declarator.id.resolve(ctx) {
        Declarator::Pointer { qualifiers, inner } => {
            let (qty, decl) = extract_declarator(sema, ctx, inner, inner_most);
            let (is_const, is_volatile) =
                constrain::declaration::check_qualifier(qualifiers).collect(sema, &declarator.span);
            let id = sema.types.pointer(qty);
            (QualifiedType::new(id, is_const, is_volatile), decl)
        }
        Declarator::Array { declarator, size } => {
            let len = size.as_ref().and_then(|e| array_length(sema, ctx, e));
            let (qty, decl) = extract_declarator(sema, ctx, declarator, inner_most);
            let id = sema.types.array(qty, len);
            (QualifiedType::new(id, false, false), decl)
        }
        _ => (inner_most, declarator.clone()),
    }
}

fn array_length(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<u32> {
    let value = ice::eval_constant(sema, ctx, expr)?;
    match value.get_integer_value() {
        Some(len) => Some(len as u32),
        None => sema.add_diag(Diag::none_diag(Diagnosis::NonIntArraySize), &expr.span),
    }
}

pub fn resolve_struct_or_union(
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

    let mut members = Vec::new();
    for field in fields {
        let qual = resolve_type(sema, ctx, &field.specifiers, &field.span);
        for declarator in &field.struct_declarators {
            let decl = &declarator.declarator;
            let Some((ty, node)) = make_qualified_type(sema, ctx, qual, decl) else { continue };
            let Some(name) = node.ident(ctx) else { continue };
            if members.iter().any(|&m| sema.symbols.get(m).name.id == name.id) {
                sema.add_diag(
                    Diag::only_diag(Diagnosis::DuplicateDeclaration(SymbolKind::Member, name)),
                    &decl.span,
                );
                continue;
            }
            let bit_width = match &declarator.bit_width {
                Some(expr) => {
                    let value = ice::eval_constant(sema, ctx, expr);
                    constrain::declaration::check_bit_width(sema.types.get(ty.ty), value).collect(sema, span)
                }
                None => None,
            };
            let memb = Symbol::member(name, ty, bit_width);
            members.push(sema.symbols.alloc(memb))
        }
    }
    sema.tags.complete(tag, members);
    tag
}

pub fn resolve_enum(sema: &mut Sema, ctx: &Context, id: EnumId) -> Option<TagDefId> {
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
            sema.add_diag(Diag::only_diag(Diagnosis::VariantBadValue), &variant.span);
            value = 0
        }
        let ty = QualifiedType::new(sema.types.int(), false, false);
        members.push(sema.declare(Symbol::variant(variant.name, ty, value as i32), &variant.span));
        value += 1;
    }
    sema.tags.complete(tag, members);
    Some(tag)
}

fn variant_value(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<i64> {
    let value = ice::eval_constant(sema, ctx, expr)?;
    match value.get_integer_value() {
        Some(v) => Some(v as i64),
        None => sema.add_diag(Diag::none_diag(Diagnosis::NonIntegerConstantExpression), &expr.span),
    }
}
