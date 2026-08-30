use crate::ast::{
    DeclarationSpecifier, Declarator, DeclaratorNode, EnumId, ExpressionNode, FunctionParameters,
    FunctionParametersNode, Name, ParameterDeclaration, StructDeclaration, Tag, TypeSpecifier,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{
    DeclaredParams, Diag, DiagCollector, Diagnosis, Member, ParamInfo, QualifiedType, ResolvedType, Sema, Symbol,
    SymbolKind, TagDefId, constrain, ice,
};

/// 6.5.2 Type specifiers
/// Each list of type specifiers shall be one of the following sets...
pub fn base_type(
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
            let tag = struct_or_union_tag(sema, ctx, Tag::Struct, node.name, &node.fields, &node.span);
            sema.types.tag(tag)
        }
        [TypeSpecifier::Union(t)] => {
            let node = t.resolve(ctx);
            let tag = struct_or_union_tag(sema, ctx, Tag::Union, node.name, &node.fields, &node.span);
            sema.types.tag(tag)
        }
        [TypeSpecifier::Enum(t)] => {
            let tag = enum_tag(sema, ctx, *t)?;
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

pub fn declared_type(
    sema: &mut Sema,
    ctx: &Context,
    inner_most: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode)> {
    let (ty, declarator, _) = extract_declarator(sema, ctx, decl, inner_most?);
    Some((ty, declarator))
}

pub fn declared_function(
    sema: &mut Sema,
    ctx: &Context,
    inner_most: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode, Option<DeclaredParams>)> {
    Some(extract_declarator(sema, ctx, decl, inner_most?))
}

fn extract_declarator(
    sema: &mut Sema,
    ctx: &Context,
    declarator: &DeclaratorNode,
    inner_most: QualifiedType,
) -> (QualifiedType, DeclaratorNode, Option<DeclaredParams>) {
    match declarator.id.resolve(ctx) {
        Declarator::Pointer { qualifiers, inner } => {
            let (is_const, is_volatile) =
                constrain::declaration::check_qualifier(qualifiers).collect(sema, &declarator.span);
            let id = sema.types.pointer(inner_most);
            extract_declarator(sema, ctx, inner, QualifiedType::new(id, is_const, is_volatile))
        }
        Declarator::Array {
            declarator: inner,
            size,
        } => {
            let len = size.as_ref().and_then(|e| array_length(sema, ctx, e));
            let id = sema.types.array(inner_most, len);
            extract_declarator(sema, ctx, inner, QualifiedType::new(id, false, false))
        }
        Declarator::Function {
            declarator: inner,
            params,
        } => {
            let list = resolve_params(sema, ctx, params);
            let id = sema.types.function(inner_most, list.types());
            let (ty, leaf, inner_list) = extract_declarator(sema, ctx, inner, QualifiedType::new(id, false, false));
            match inner.id.resolve(ctx) {
                Declarator::Ident(_) | Declarator::Abstract => (ty, leaf, Some(list)),
                _ => (ty, leaf, inner_list),
            }
        }
        _ => (inner_most, declarator.clone(), None),
    }
}

fn resolve_params(sema: &mut Sema, ctx: &Context, params: &FunctionParametersNode) -> DeclaredParams {
    match &params.param {
        FunctionParameters::Empty => DeclaredParams::Unspecified,
        FunctionParameters::OldStyle(names) => DeclaredParams::Names(names.clone()),
        FunctionParameters::ParameterTypeList(params) => resolve_prototype(sema, ctx, params, false),
        FunctionParameters::Variadic(params) => resolve_prototype(sema, ctx, params, true),
    }
}

/// 6.5.4.3 Function declarators
/// The special case of an unnamed parameter of type void as the only item in the list specifies
/// that the function has no parameters.
fn resolve_prototype(
    sema: &mut Sema,
    ctx: &Context,
    params: &[ParameterDeclaration],
    is_variadic: bool,
) -> DeclaredParams {
    if let [only] = params
        && !is_variadic
        && only.is_abstract_void(ctx)
    {
        return DeclaredParams::Prototype {
            params: Vec::new(),
            is_variadic,
        };
    }
    let params: Vec<ParamInfo> = params
        .iter()
        .filter_map(|param| resolve_parameter(sema, ctx, param))
        .collect();
    for param in &params {
        let is_void = matches!(sema.types.get(param.ty.id), ResolvedType::Void);
        let is_special_case = params.len() == 1 && param.name.is_some();
        constrain::external::check_void_parameter(is_void && !is_special_case).collect(sema, &param.span);
    }
    DeclaredParams::Prototype { params, is_variadic }
}

fn resolve_parameter(sema: &mut Sema, ctx: &Context, param: &ParameterDeclaration) -> Option<ParamInfo> {
    let span = &param.span;
    let qualif = base_type(sema, ctx, &param.specifiers, span);
    let (ty, decl) = declared_type(sema, ctx, qualif, &param.declarator)?;
    let ty = sema.types.adjust_parameter(ty);
    let storage = constrain::declaration::get_storage(&param.specifiers).collect(sema, span);
    if let Some(storage) = storage {
        constrain::external::param_storage_only_register(storage).collect(sema, span);
    }
    Some(ParamInfo {
        name: decl.ident(ctx),
        ty,
        storage,
        span: *span,
    })
}

fn array_length(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<usize> {
    let value = ice::eval_constant(sema, ctx, expr)?;
    match value.get_integer_value() {
        Some(len) => Some(len as usize),
        None => sema.add_diag(Diag::none_diag(Diagnosis::NonIntArraySize), &expr.span),
    }
}

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
    for field in fields {
        if field.struct_declarators.is_empty() {
            sema.add_diag(Diag::only_diag(Diagnosis::EmptyDeclaration), &field.span);
        }
        let qual = base_type(sema, ctx, &field.specifiers, &field.span);
        for declarator in &field.struct_declarators {
            let decl = &declarator.declarator;
            let Some((ty, node)) = declared_type(sema, ctx, qual, decl) else { continue };
            let bit_width = declarator.bit_width.as_ref().and_then(|e| {
                let value = ice::eval_constant(sema, ctx, e);
                constrain::declaration::check_bit_width(sema.types.get(ty.id), value).collect(sema, span)
            });
            match (node.ident(ctx), bit_width) {
                (Some(name), _) => {
                    if members
                        .iter()
                        .any(|&m| matches!(m, Member::Symbol(id) if sema.symbols.get(id).name.id == name.id ))
                    {
                        sema.add_diag(
                            Diag::only_diag(Diagnosis::DuplicateDeclaration(SymbolKind::Member, name)),
                            &decl.span,
                        );
                        continue;
                    }
                    let memb = Symbol::member(name, ty, bit_width);
                    let id = sema.symbols.alloc(memb);
                    members.push(Member::Symbol(id))
                }
                (None, Some(i)) if bit_width.is_some() => members.push(Member::Bitfield(i)),
                _ => (),
            }
        }
    }
    if members.is_empty() || members.iter().all(|m| matches!(m, Member::Bitfield(_))) {
        sema.add_diag(Diag::only_diag(Diagnosis::TagWithoutMember(kind.symbol_kind())), span);
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
            sema.add_diag(Diag::only_diag(Diagnosis::VariantBadValue), &variant.span);
            value = 0
        }
        let ty = QualifiedType::new(sema.builtins.int, false, false);
        members.push(Member::Symbol(
            sema.declare(Symbol::variant(variant.name, ty, value as i32), &variant.span),
        ));
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
