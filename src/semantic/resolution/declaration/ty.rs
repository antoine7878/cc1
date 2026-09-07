use crate::arena::ResolveWith;
use crate::ast::{
    DeclarationSpecifier, Declarator, DeclaratorNode, ExpressionNode, FunctionParameters, FunctionParametersNode,
    ParameterDeclaration, Tag, TypeSpecifier,
};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    DeclaredParams, Diag, DiagCollector, Diagnosis, ParamInfo, QualifiedType, ResolvedType, Sema, constrain, ice,
};

pub fn base_type(
    sema: &mut Sema,
    ctx: &Context,
    specifiers: &[DeclarationSpecifier],
    span: &Span,
) -> Option<QualifiedType> {
    let (is_const, is_volatile) = constrain::specifier::get_qualifier(specifiers).collect(sema, span);

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
            let ty = constrain::specifier::basic_type(s).collect(sema, span)?;
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
    let (ty, declarator, _) = extract_declarator(sema, ctx, decl, inner_most?, false);
    Some((ty, declarator))
}

pub fn declared_function(
    sema: &mut Sema,
    ctx: &Context,
    inner_most: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode, Option<DeclaredParams>)> {
    Some(extract_declarator(sema, ctx, decl, inner_most?, false))
}

fn extract_declarator(
    sema: &mut Sema,
    ctx: &Context,
    declarator: &DeclaratorNode,
    inner_most: QualifiedType,
    inner_already_diagnosed: bool,
) -> (QualifiedType, DeclaratorNode, Option<DeclaredParams>) {
    match declarator.id.resolve(ctx) {
        Declarator::Pointer { qualifiers, inner } => {
            let (is_const, is_volatile) =
                constrain::specifier::check_qualifier(qualifiers.iter().copied()).collect(sema, &declarator.span);
            let id = sema.types.pointer(inner_most);
            extract_declarator(sema, ctx, inner, QualifiedType::new(id, is_const, is_volatile), false)
        }
        Declarator::Array {
            declarator: inner,
            size,
        } => {
            if !inner_already_diagnosed {
                constrain::ty::check_element_type(inner_most.is_object(sema), inner_most)
                    .collect(sema, &declarator.span);
            }
            let len = size.as_ref().and_then(|e| array_length(sema, ctx, e));
            let this_level_erred = size.is_some() && len.is_none();
            let id = sema.types.array(inner_most, len);
            extract_declarator(sema, ctx, inner, QualifiedType::plain(id), this_level_erred)
        }
        Declarator::Function {
            declarator: inner,
            params,
        } => {
            let list = resolve_params(sema, ctx, params);
            constrain::ty::check_return_type(inner_most.id.resolve(sema), inner_most).collect(sema, &declarator.span);
            let id = sema.types.function(inner_most, list.types());
            let (ty, leaf, inner_list) = extract_declarator(sema, ctx, inner, QualifiedType::plain(id), false);
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
        let is_void = matches!(param.ty.id.resolve(sema), ResolvedType::Void);
        let is_special_case = params.len() == 1 && param.name.is_some();
        constrain::parameter::check_void_parameter(is_void && !is_special_case).collect(sema, &param.span);
    }
    DeclaredParams::Prototype { params, is_variadic }
}

fn resolve_parameter(sema: &mut Sema, ctx: &Context, param: &ParameterDeclaration) -> Option<ParamInfo> {
    let span = &param.span;
    let qualif = base_type(sema, ctx, &param.specifiers, span);
    let (ty, decl) = declared_type(sema, ctx, qualif, &param.declarator)?;
    let ty = sema.types.adjust_parameter(ty);
    let storage = constrain::specifier::get_storage(&param.specifiers).collect(sema, span);
    if let Some(storage) = storage {
        constrain::parameter::param_storage_only_register(storage).collect(sema, span);
    }
    Some(ParamInfo {
        name: decl.ident(ctx),
        ty,
        storage,
        span: *span,
    })
}

/// 6.5.4.2 The expression delimited by [ and ] (which specifies the size of an array) shall be an
/// integral constant expression that has a value greater than zero.
fn array_length(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Option<usize> {
    let value = ice::eval_constant(sema, ctx, expr)?;
    let Some(len) = value.get_integer_value() else {
        return sema.add_diag(Diag::err(None, Diagnosis::NonIntArraySize), &expr.span);
    };
    // `get_integer_value` reinterprets the representation, so the sign is read off the value.
    if value.is_negative() {
        return sema.add_diag(Diag::err(None, Diagnosis::NegativeArraySize), &expr.span);
    }
    if value.is_zero() {
        return sema.add_diag(Diag::err(None, Diagnosis::ZeroArraySize), &expr.span);
    }
    Some(len as usize)
}
