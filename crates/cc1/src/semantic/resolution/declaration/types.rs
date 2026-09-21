use libft::Span;

use crate::ast::{
    DeclarationSpecifier, Declarator, DeclaratorNode, ExpressionNode, FunctionParameters, FunctionParametersNode,
    ParameterDeclaration, Tag, TypeSpecifier,
};
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    DeclaredParams, Diag, Diagnostic, DiagnosticSink, ParamInfo, QualifiedType, ResolvedType, Resolver, constraints,
    layout,
};

pub fn base_type(resolver: &mut Resolver, specifiers: &[DeclarationSpecifier], span: &Span) -> Option<QualifiedType> {
    let (is_const, is_volatile) = constraints::specifier::qualifiers_of(specifiers).collect(resolver, span);

    let types: Vec<_> = specifiers
        .iter()
        .filter_map(|s| match s {
            DeclarationSpecifier::Type(t) => Some(t),
            _ => None,
        })
        .collect();

    let id = match types.as_slice() {
        [TypeSpecifier::Struct(t)] => {
            let node = t.resolve();
            let tag = struct_or_union_tag(resolver, Tag::Struct, node.name, &node.declarations, &node.span);
            resolver.sema.types.tag(tag)
        }
        [TypeSpecifier::Union(t)] => {
            let node = t.resolve();
            let tag = struct_or_union_tag(resolver, Tag::Union, node.name, &node.declarations, &node.span);
            resolver.sema.types.tag(tag)
        }
        [TypeSpecifier::Enum(t)] => {
            let tag = enum_tag(resolver, *t)?;
            resolver.sema.types.tag(tag)
        }
        [TypeSpecifier::TypedefName(t)] => return resolver.resolve_typedef(*t, is_const, is_volatile, span),
        s => {
            let ty = constraints::specifier::basic_type(s).collect(resolver, span)?;
            resolver.sema.types.intern(ty)
        }
    };
    Some(QualifiedType::new(id, is_const, is_volatile))
}

pub fn declared_type(
    resolver: &mut Resolver,
    inner_most: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode)> {
    let (ty, declarator, _) = extract_declarator(resolver, decl, inner_most?, false);
    Some((ty, declarator))
}

pub fn declared_function(
    resolver: &mut Resolver,
    inner_most: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode, Option<DeclaredParams>)> {
    Some(extract_declarator(resolver, decl, inner_most?, false))
}

fn extract_declarator(
    resolver: &mut Resolver,
    declarator: &DeclaratorNode,
    inner_most: QualifiedType,
    inner_already_diagnosed: bool,
) -> (QualifiedType, DeclaratorNode, Option<DeclaredParams>) {
    match declarator.id.resolve() {
        Declarator::Pointer { qualifiers, inner } => {
            let (is_const, is_volatile) = constraints::specifier::check_qualifiers(qualifiers.iter().copied())
                .collect(resolver, &declarator.span);
            let id = resolver.sema.types.pointer(inner_most);
            extract_declarator(resolver, inner, QualifiedType::new(id, is_const, is_volatile), false)
        }
        Declarator::Array { declarator: inner, size } => {
            if !inner_already_diagnosed {
                constraints::types::check_element_type(inner_most.is_object(resolver.sema), inner_most)
                    .collect(resolver, &declarator.span);
            }
            let len = size.as_ref().and_then(|e| array_length(resolver, e));
            let len = len.and_then(|len| array_size(resolver, inner_most, len, &declarator.span));
            let this_level_erred = size.is_some() && len.is_none();
            let id = resolver.sema.types.array(inner_most, len);
            extract_declarator(resolver, inner, QualifiedType::plain(id), this_level_erred)
        }
        Declarator::Function { declarator: inner, params } => {
            let list = resolve_params(resolver, params);
            constraints::types::check_return_type(inner_most.id.resolve_with(resolver.sema), inner_most)
                .collect(resolver, &declarator.span);
            let id = resolver.sema.types.function(inner_most, list.types());
            let (ty, leaf, inner_list) = extract_declarator(resolver, inner, QualifiedType::plain(id), false);
            match inner.id.resolve() {
                Declarator::Ident(_) | Declarator::Abstract => (ty, leaf, Some(list)),
                _ => (ty, leaf, inner_list),
            }
        }
        _ => (inner_most, declarator.clone(), None),
    }
}

fn resolve_params(resolver: &mut Resolver, params: &FunctionParametersNode) -> DeclaredParams {
    match &params.param {
        FunctionParameters::Empty => DeclaredParams::Unspecified,
        FunctionParameters::OldStyle(names) => DeclaredParams::Names(names.clone()),
        FunctionParameters::ParameterTypeList(params) => resolve_prototype(resolver, params, false),
        FunctionParameters::Variadic(params) => resolve_prototype(resolver, params, true),
    }
}

fn resolve_prototype(resolver: &mut Resolver, params: &[ParameterDeclaration], is_variadic: bool) -> DeclaredParams {
    if let [only] = params
        && !is_variadic
        && only.is_abstract_void()
    {
        return DeclaredParams::Prototype { params: Vec::new(), is_variadic };
    }
    let params: Vec<ParamInfo> = params.iter().filter_map(|param| resolve_param(resolver, param)).collect();
    for param in &params {
        let is_void = matches!(param.ty.id.resolve_with(resolver.sema), ResolvedType::Void);
        let is_special_case = params.len() == 1 && param.name.is_some();
        constraints::param::check_void_param(is_void && !is_special_case).collect(resolver, &param.span);
    }
    DeclaredParams::Prototype { params, is_variadic }
}

fn resolve_param(resolver: &mut Resolver, param: &ParameterDeclaration) -> Option<ParamInfo> {
    let span = &param.span;
    let qualif = base_type(resolver, &param.specifiers, span);
    let (ty, decl) = declared_type(resolver, qualif, &param.declarator)?;
    let ty = resolver.sema.types.adjust_param(ty);
    let storage = constraints::specifier::storage_of(&param.specifiers).collect(resolver, span);
    if let Some(storage) = storage {
        constraints::param::check_param_storage(storage).collect(resolver, span);
    }
    Some(ParamInfo { name: decl.name(), ty, storage, span: *span })
}

fn array_size(resolver: &mut Resolver, elem: QualifiedType, len: usize, span: &Span) -> Option<usize> {
    let Some(layout) = layout::of(resolver.sema, elem.id) else { return Some(len) };
    let size = u64::from(layout.size).saturating_mul(len as u64);
    if size > i32::MAX as u64 {
        return resolver.add_diag(Diag::err(None, Diagnostic::ArrayTooLarge(size)), span);
    }
    Some(len)
}

fn array_length(resolver: &mut Resolver, expr: &ExpressionNode) -> Option<usize> {
    let value = resolver.eval_constant(expr)?;
    let Some(len) = value.get_integer_value() else {
        return resolver.add_diag(Diag::err(None, Diagnostic::NonIntArraySize), &expr.span);
    };
    if value.is_negative() {
        return resolver.add_diag(Diag::err(None, Diagnostic::NegativeArraySize), &expr.span);
    }
    if value.is_zero() {
        return resolver.add_diag(Diag::err(None, Diagnostic::ZeroArraySize), &expr.span);
    }
    Some(len as usize)
}
