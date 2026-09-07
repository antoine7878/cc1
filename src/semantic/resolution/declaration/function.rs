use crate::arena::ResolveWith;
use crate::ast::{DeclarationNode, DeclarationSpecifier, DeclaratorNode, FunctionDefinitionNode, Name, Storage};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    DeclaredParams, Definition, Diag, DiagCollector, Diagnosis, FunctionDefId, ParamInfo, ParamTypes, QualifiedType,
    ResolvedType, ScopeKind, Sema, Symbol, SymbolId, SymbolKind, constrain,
};

pub struct FunctionHeader {
    pub id: FunctionDefId,
    pub params: DeclaredParams,
    pub declared: Option<ParamTypes>,
    pub return_ty: QualifiedType,
}

pub fn bind_function_parameters(
    sema: &mut Sema,
    ctx: &Context,
    node: &FunctionDefinitionNode,
    header: &FunctionHeader,
) {
    sema.scopes.push(ScopeKind::Prototype);
    let lst = &node.old_style_declarations;
    let span = &node.declarator.span;
    let parameters = match &header.params {
        DeclaredParams::Unspecified => param_empty(sema, lst, span),
        DeclaredParams::Names(names) => param_old_style(sema, ctx, names, lst, span),
        DeclaredParams::Prototype { params, .. } => param_prototype(sema, params, lst, span),
    };
    if let Some(declared) = &header.declared
        && !matches!(header.params, DeclaredParams::Prototype { .. })
        && let Some(name) = node.declarator.ident(ctx)
    {
        check_identifier_list(sema, declared, &header.params, &parameters, name, span);
    }
    sema.functions.complete(header.id, parameters);
}

pub fn define_function(sema: &mut Sema, ctx: &Context, node: &FunctionDefinitionNode) -> Option<FunctionHeader> {
    let span = &node.span;
    let decl_span = &node.declarator.span;

    let qualif = base_type(sema, ctx, &node.specifiers, span);
    let (rty, decl, params) = declared_function(sema, ctx, qualif, &node.declarator)?;
    let params = constrain::ty::extract_function_declarator(params).collect(sema, decl_span)?;

    let declared_storage = constrain::specifier::get_storage(&node.specifiers).collect(sema, span);
    let storage = declared_storage.unwrap_or(Storage::Extern);

    constrain::specifier::check_function_storage(storage).collect(sema, span);
    constrain::specifier::check_external_specifiers(&node.specifiers).collect(sema, span);

    let name = decl.ident(ctx)?;
    let previous = sema.scopes.current(SymbolKind::Function, name.id);
    let declared = previous.and_then(|id| param_types(sema, id));
    let ty = match &declared {
        Some(declared) if !matches!(params, DeclaredParams::Prototype { .. }) => {
            with_param_types(sema, rty, declared.clone())
        }
        _ => rty,
    };
    let &ResolvedType::Function { ret: return_ty, .. } = ty.id.resolve(sema) else { unreachable!() };
    constrain::ty::check_definition_return(return_ty.is_void(sema) || return_ty.is_complete(sema), return_ty)
        .collect(sema, decl_span);
    let prior = sema.linkage_of_name(name.id);
    let linkage = Symbol::linkage_of(sema.scopes.kind(), declared_storage, SymbolKind::Function, prior);
    let mut sym = Symbol::function(name, ty, storage);
    sym.linkage = linkage;
    sym.definition = Definition::Definition;
    let sym = sema.declare(sym, decl_span);
    let declared = (previous == Some(sym)).then_some(declared).flatten();
    Some(FunctionHeader {
        id: sema.functions.declare(sym),
        params,
        declared,
        return_ty,
    })
}

fn param_types(sema: &Sema, sym: SymbolId) -> Option<ParamTypes> {
    let ty = sym.resolve(sema).ty?;
    match ty.id.resolve(sema) {
        ResolvedType::Function { params, .. } => Some(params.clone()),
        _ => None,
    }
}

fn with_param_types(sema: &mut Sema, ty: QualifiedType, params: ParamTypes) -> QualifiedType {
    let ret = match ty.id.resolve(sema) {
        ResolvedType::Function { ret, .. } => *ret,
        _ => return ty,
    };
    QualifiedType::new(sema.types.function(ret, params), ty.is_const, ty.is_volatile)
}

fn check_identifier_list(
    sema: &mut Sema,
    declared: &ParamTypes,
    params: &DeclaredParams,
    parameters: &[SymbolId],
    name: Name,
    span: &Span,
) {
    if let DeclaredParams::Names(names) = params
        && names.len() != parameters.len()
    {
        return;
    }
    let identifiers: Vec<QualifiedType> = parameters.iter().filter_map(|sym| (*sym).resolve(sema).ty).collect();
    if identifiers.len() != parameters.len() || declared.is_compatible_with_identifiers(sema, &identifiers) {
        return;
    }
    sema.add_diag(
        Diag::err((), Diagnosis::DuplicateDeclaration(SymbolKind::Function, name)),
        span,
    );
}

fn param_empty(sema: &mut Sema, lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
    if !lst.is_empty() {
        sema.add_diag(Diag::err((), Diagnosis::ParameterTypeListWithList), span)
    }
    Vec::new()
}

fn param_prototype(sema: &mut Sema, params: &[ParamInfo], lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
    if !constrain::parameter::is_valid_parameter_style(params, lst).collect(sema, span) {
        return Vec::new();
    }
    if let [only] = params {
        let is_void = matches!(only.ty.id.resolve(sema), ResolvedType::Void);
        constrain::parameter::check_void_parameter(is_void).collect(sema, &only.span);
    }
    for param in params {
        if param.ty.is_void(sema) {
            continue;
        }
        constrain::parameter::check_complete_parameter(param.ty.is_complete(sema), param.ty).collect(sema, &param.span);
    }
    params.iter().filter_map(|param| add_parameter(sema, param)).collect()
}

fn add_parameter(sema: &mut Sema, param: &ParamInfo) -> Option<SymbolId> {
    let name = param.name?;
    let storage = param.storage.unwrap_or(Storage::Auto);
    let sym = Symbol::parameter(name, param.ty, storage);
    Some(sema.declare(sym, &name.span))
}

fn param_old_style(
    sema: &mut Sema,
    ctx: &Context,
    names: &[Name],
    lst: &[DeclarationNode],
    span: &Span,
) -> Vec<SymbolId> {
    let declarations: Vec<_> = lst
        .iter()
        .flat_map(
            |DeclarationNode {
                 span,
                 specifiers,
                 init_declarators,
             }| {
                init_declarators
                    .iter()
                    .map(|decl| {
                        add_parameter_declarator(sema, ctx, specifiers, &decl.declarator, span)
                            .map(|sym_id| sym_id.resolve(sema).name.id)
                    })
                    .collect::<Vec<_>>()
            },
        )
        .collect();

    let names_id: Vec<_> = names.iter().map(|n| n.id).collect();

    let Some(missing_id) = constrain::parameter::is_valid_old_style(&names_id, declarations).collect(sema, span) else {
        return Vec::new();
    };
    let ty = QualifiedType::plain(sema.builtins.int);
    missing_id
        .into_iter()
        .map(|string_id| Name::new(string_id, Span::default()))
        .map(|name| Symbol::parameter(name, ty, Storage::Auto))
        .for_each(|sym| {
            let _ = sema.declare(sym, &Span::default());
        });
    names
        .iter()
        .filter_map(|name| sema.scopes.lookup_ordinary(name.id))
        .collect()
}

fn add_parameter_declarator(
    sema: &mut Sema,
    ctx: &Context,
    specifiers: &[DeclarationSpecifier],
    decl: &DeclaratorNode,
    span: &Span,
) -> Option<SymbolId> {
    let qualif = base_type(sema, ctx, specifiers, span);
    let (ty, decl) = declared_type(sema, ctx, qualif, decl)?;
    let ty = sema.types.adjust_parameter(ty);
    let declared_storage = constrain::specifier::get_storage(specifiers).collect(sema, span);
    if let Some(storage) = declared_storage {
        constrain::parameter::param_storage_only_register(storage).collect(sema, span)?;
    }
    let name = decl.ident(ctx)?;
    if !ty.is_void(sema) {
        constrain::parameter::check_complete_parameter(ty.is_complete(sema), ty).collect(sema, &decl.span);
    }
    let storage = declared_storage.unwrap_or(Storage::Auto);
    let sym = Symbol::parameter(name, ty, storage);
    Some(sema.declare(sym, &decl.span))
}

pub fn implicit_declare_function(sema: &mut Sema, fn_name: &Name, span: &Span) {
    let ret = QualifiedType::plain(sema.builtins.int);
    let fn_ty = sema.types.function(ret, ParamTypes::Unspecified);
    let ty = QualifiedType::plain(fn_ty);
    let sym = Symbol::function(*fn_name, ty, Storage::Extern);
    sema.declare(sym, span);
}
