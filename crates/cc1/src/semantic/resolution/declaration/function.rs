use libft::Span;

use crate::ast::{DeclarationNode, DeclarationSpecifier, DeclaratorNode, FunctionDefinitionNode, Name, Storage};
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    DeclaredParams, Definition, Diag, DiagCollector, Diagnosis, FunctionDefId, ParamInfo, ParamTypes, QualifiedType,
    ResolvedType, Sema, Symbol, SymbolId, SymbolKind, SymbolResolver, constrain,
};

#[derive(Debug, Clone)]
pub struct FunctionHeader {
    pub id: FunctionDefId,
    pub params: DeclaredParams,
    pub declared: Option<ParamTypes>,
}

pub fn bind_function_parameters(resolver: &mut SymbolResolver, node: &FunctionDefinitionNode, header: &FunctionHeader) {
    resolver.enter_prototype();
    let lst = &node.old_style_declarations;
    let span = &node.declarator.span;
    let parameters = match &header.params {
        DeclaredParams::Unspecified => param_empty(resolver.sema, lst, span),
        DeclaredParams::Names(names) => param_old_style(resolver, names, lst, span),
        DeclaredParams::Prototype { params, .. } => param_prototype(resolver, params, lst, span),
    };
    if let Some(declared) = &header.declared
        && !matches!(header.params, DeclaredParams::Prototype { .. })
        && let Some(name) = node.declarator.ident()
    {
        check_identifier_list(resolver.sema, declared, &header.params, &parameters, name, span);
    }
    resolver.sema.functions.complete(header.id, parameters);
}

pub fn define_function(resolver: &mut SymbolResolver, node: &FunctionDefinitionNode) -> Option<FunctionHeader> {
    let span = &node.span;
    let decl_span = &node.declarator.span;

    let qualif = base_type(resolver, &node.specifiers, span);
    let (rty, decl, params) = declared_function(resolver, qualif, &node.declarator)?;
    let params = constrain::ty::extract_function_declarator(params).collect(resolver, decl_span)?;

    let declared_storage = constrain::specifier::get_storage(&node.specifiers).collect(resolver, span);
    let storage = declared_storage.unwrap_or(Storage::Extern);

    constrain::specifier::check_function_storage(storage).collect(resolver, span);
    constrain::specifier::check_external_specifiers(&node.specifiers).collect(resolver, span);

    let name = decl.ident()?;
    let previous = resolver.current(name.id);
    let declared = previous.and_then(|id| param_types(resolver.sema, id));
    let ty = match &declared {
        Some(declared) if !matches!(params, DeclaredParams::Prototype { .. }) => {
            with_param_types(resolver.sema, rty, declared.clone())
        }
        _ => rty,
    };
    let &ResolvedType::Function { ret: return_ty, .. } = ty.id.resolve_with(resolver.sema) else { unreachable!() };
    let is_defined_return = return_ty.is_void(resolver.sema) || return_ty.is_complete(resolver.sema);
    constrain::ty::check_definition_return(is_defined_return, return_ty).collect(resolver, decl_span);
    let prior = resolver.sema.linkage_of_name(name.id);
    let linkage = Symbol::linkage_of(resolver.scope_kind(), declared_storage, SymbolKind::Function, prior);
    let mut sym = Symbol::function(name, ty, storage);
    sym.linkage = linkage;
    sym.definition = Definition::Definition;
    let sym = resolver.declare(sym, decl_span);
    let declared = (previous == Some(sym)).then_some(declared).flatten();
    Some(FunctionHeader { id: resolver.sema.functions.declare(sym, return_ty), params, declared })
}

fn param_types(sema: &Sema, sym: SymbolId) -> Option<ParamTypes> {
    let ty = sym.resolve_with(sema).ty;
    match ty.id.resolve_with(sema) {
        ResolvedType::Function { params, .. } => Some(params.clone()),
        _ => None,
    }
}

fn with_param_types(sema: &mut Sema, ty: QualifiedType, params: ParamTypes) -> QualifiedType {
    let ret = match ty.id.resolve_with(sema) {
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
    let identifiers: Vec<QualifiedType> = parameters.iter().map(|sym| (*sym).resolve_with(sema).ty).collect();
    if declared.is_compatible_with_identifiers(sema, &identifiers) {
        return;
    }
    sema.add_diag(Diag::err((), Diagnosis::DuplicateDeclaration(SymbolKind::Function, name)), span);
}

fn param_empty(sema: &mut Sema, lst: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
    if !lst.is_empty() {
        sema.add_diag(Diag::err((), Diagnosis::ParameterTypeListWithList), span)
    }
    Vec::new()
}

fn param_prototype(
    resolver: &mut SymbolResolver,
    params: &[ParamInfo],
    lst: &[DeclarationNode],
    span: &Span,
) -> Vec<SymbolId> {
    if !constrain::parameter::is_valid_parameter_style(params, lst).collect(resolver, span) {
        return Vec::new();
    }
    if let [only] = params {
        let is_void = matches!(only.ty.id.resolve_with(resolver.sema), ResolvedType::Void);
        constrain::parameter::check_void_parameter(is_void).collect(resolver, &only.span);
    }
    for param in params {
        if param.ty.is_void(resolver.sema) {
            continue;
        }
        constrain::parameter::check_complete_parameter(param.ty.is_complete(resolver.sema), param.ty)
            .collect(resolver, &param.span);
    }
    params.iter().filter_map(|param| add_parameter(resolver, param)).collect()
}

fn add_parameter(resolver: &mut SymbolResolver, param: &ParamInfo) -> Option<SymbolId> {
    let name = param.name?;
    let storage = param.storage.unwrap_or(Storage::Auto);
    let sym = Symbol::parameter(name, param.ty, storage);
    Some(resolver.declare(sym, &name.span))
}

fn param_old_style(
    resolver: &mut SymbolResolver,
    names: &[Name],
    lst: &[DeclarationNode],
    span: &Span,
) -> Vec<SymbolId> {
    let declarations: Vec<_> = lst
        .iter()
        .flat_map(|DeclarationNode { span, specifiers, init_declarators }| {
            init_declarators
                .iter()
                .map(|decl| {
                    add_parameter_declarator(resolver, specifiers, &decl.declarator, span)
                        .map(|sym_id| sym_id.resolve_with(resolver.sema).name.id)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let names_id: Vec<_> = names.iter().map(|n| n.id).collect();

    let Some(missing_id) = constrain::parameter::is_valid_old_style(&names_id, declarations).collect(resolver, span)
    else {
        return Vec::new();
    };
    let ty = QualifiedType::plain(resolver.sema.builtins.int);
    missing_id
        .into_iter()
        .map(|string_id| Name::new(string_id, Span::default()))
        .map(|name| Symbol::parameter(name, ty, Storage::Auto))
        .for_each(|sym| {
            let _ = resolver.declare(sym, &Span::default());
        });
    names.iter().filter_map(|name| resolver.lookup_ordinary(name.id)).collect()
}

fn add_parameter_declarator(
    resolver: &mut SymbolResolver,
    specifiers: &[DeclarationSpecifier],
    decl: &DeclaratorNode,
    span: &Span,
) -> Option<SymbolId> {
    let qualif = base_type(resolver, specifiers, span);
    let (ty, decl) = declared_type(resolver, qualif, decl)?;
    let ty = resolver.sema.types.adjust_parameter(ty);
    let declared_storage = constrain::specifier::get_storage(specifiers).collect(resolver, span);
    if let Some(storage) = declared_storage {
        constrain::parameter::param_storage_only_register(storage).collect(resolver, span)?;
    }
    let name = decl.ident()?;
    if !ty.is_void(resolver.sema) {
        constrain::parameter::check_complete_parameter(ty.is_complete(resolver.sema), ty).collect(resolver, &decl.span);
    }
    let storage = declared_storage.unwrap_or(Storage::Auto);
    let sym = Symbol::parameter(name, ty, storage);
    Some(resolver.declare(sym, &decl.span))
}

pub fn implicit_declare_function(resolver: &mut SymbolResolver, fn_name: &Name, span: &Span) {
    let ret = QualifiedType::plain(resolver.sema.builtins.int);
    let fn_ty = resolver.sema.types.function(ret, ParamTypes::Unspecified);
    let ty = QualifiedType::plain(fn_ty);
    let sym = Symbol::function(*fn_name, ty, Storage::Extern);
    resolver.declare(sym, span);
}
