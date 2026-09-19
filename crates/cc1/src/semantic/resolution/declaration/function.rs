use libft::Span;

use crate::ast::{DeclarationNode, DeclarationSpecifier, DeclaratorNode, FunctionDefinitionNode, Name, Storage};
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    DeclaredParams, DefinitionState, Diag, Diagnostic, DiagnosticSink, FunctionDefId, ParamInfo, ParamTypes,
    QualifiedType, ResolvedType, Resolver, Sema, Symbol, SymbolId, SymbolKind, constraints,
};

#[derive(Debug, Clone)]
pub struct FunctionHeader {
    pub id: FunctionDefId,
    pub params: DeclaredParams,
    pub declared: Option<ParamTypes>,
}

impl Default for FunctionHeader {
    fn default() -> Self {
        Self { id: 0.into(), params: DeclaredParams::Unspecified, declared: None }
    }
}

pub fn bind_function_params(resolver: &mut Resolver, node: &FunctionDefinitionNode, header: &FunctionHeader) {
    resolver.enter_prototype();
    let declarations = &node.old_style_declarations;
    let span = &node.declarator.span;
    let params = match &header.params {
        DeclaredParams::Unspecified => param_empty(resolver.sema, declarations, span),
        DeclaredParams::Names(names) => param_old_style(resolver, names, declarations, span),
        DeclaredParams::Prototype { params, .. } => param_prototype(resolver, params, declarations, span),
    };
    if let Some(declared) = &header.declared
        && !matches!(header.params, DeclaredParams::Prototype { .. })
        && let Some(name) = node.declarator.name()
    {
        check_identifier_list(resolver.sema, declared, &header.params, &params, name, span);
    }
    resolver.sema.functions.complete(header.id, params);
}

pub fn define_function(resolver: &mut Resolver, node: &FunctionDefinitionNode) -> Option<FunctionHeader> {
    let span = &node.span;
    let decl_span = &node.declarator.span;

    let qualif = base_type(resolver, &node.specifiers, span);
    let (rty, decl, params) = declared_function(resolver, qualif, &node.declarator)?;
    let params = constraints::types::extract_function_declarator(params).collect(resolver, decl_span)?;

    let declared_storage = constraints::specifier::storage_of(&node.specifiers).collect(resolver, span);
    let storage = declared_storage.unwrap_or(Storage::Extern);

    constraints::specifier::check_function_storage(storage).collect(resolver, span);
    constraints::specifier::check_external_specifiers(&node.specifiers).collect(resolver, span);

    let name = decl.name()?;
    let previous = resolver.lookup_current(name.id);
    let declared = previous.and_then(|id| param_types(resolver.sema, id));
    let ty = match &declared {
        Some(declared) if !matches!(params, DeclaredParams::Prototype { .. }) => {
            with_param_types(resolver.sema, rty, declared.clone())
        }
        _ => rty,
    };
    let &ResolvedType::Function { ret: return_ty, .. } = ty.id.resolve_with(resolver.sema) else {
        unreachable!()
    };
    let is_defined_return = return_ty.is_void(resolver.sema) || return_ty.is_complete(resolver.sema);
    constraints::types::check_definition_return(is_defined_return, return_ty).collect(resolver, decl_span);
    let prior = resolver.sema.linkage_of_name(name.id);
    let linkage = Symbol::linkage_of(resolver.scope_kind(), declared_storage, SymbolKind::Function, prior);
    let mut sym = Symbol::function(name, ty, storage);
    sym.linkage = linkage;
    sym.definition = DefinitionState::Defined;
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
    declared_params: &DeclaredParams,
    symbols: &[SymbolId],
    name: Name,
    span: &Span,
) {
    if let DeclaredParams::Names(names) = declared_params
        && names.len() != symbols.len()
    {
        return;
    }
    let identifiers: Vec<QualifiedType> = symbols.iter().map(|sym| (*sym).resolve_with(sema).ty).collect();
    if declared.is_compatible_with_identifiers(sema, &identifiers) {
        return;
    }
    sema.add_diag(Diag::err((), Diagnostic::DuplicateDeclaration(SymbolKind::Function, name)), span);
}

fn param_empty(sema: &mut Sema, declarations: &[DeclarationNode], span: &Span) -> Vec<SymbolId> {
    if !declarations.is_empty() {
        sema.add_diag(Diag::err((), Diagnostic::ParameterTypeListWithList), span)
    }
    Vec::new()
}

fn param_prototype(
    resolver: &mut Resolver,
    params: &[ParamInfo],
    declarations: &[DeclarationNode],
    span: &Span,
) -> Vec<SymbolId> {
    if !constraints::param::is_valid_param_style(params, declarations).collect(resolver, span) {
        return Vec::new();
    }
    if let [only] = params {
        let is_void = matches!(only.ty.id.resolve_with(resolver.sema), ResolvedType::Void);
        constraints::param::check_void_param(is_void).collect(resolver, &only.span);
    }
    for param in params {
        if param.ty.is_void(resolver.sema) {
            continue;
        }
        constraints::param::check_complete_param(param.ty.is_complete(resolver.sema), param.ty)
            .collect(resolver, &param.span);
    }
    params.iter().filter_map(|param| add_param(resolver, param)).collect()
}

fn add_param(resolver: &mut Resolver, param: &ParamInfo) -> Option<SymbolId> {
    let name = param.name?;
    let storage = param.storage.unwrap_or(Storage::Auto);
    let sym = Symbol::param(name, param.ty, storage);
    Some(resolver.declare(sym, &name.span))
}

fn param_old_style(
    resolver: &mut Resolver,
    names: &[Name],
    declarations: &[DeclarationNode],
    span: &Span,
) -> Vec<SymbolId> {
    let declared_names: Vec<_> = declarations
        .iter()
        .flat_map(|DeclarationNode { span, specifiers, init_declarators }| {
            init_declarators
                .iter()
                .map(|decl| {
                    add_param_declarator(resolver, specifiers, &decl.declarator, span)
                        .map(|sym_id| sym_id.resolve_with(resolver.sema).name.id)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let names_id: Vec<_> = names.iter().map(|n| n.id).collect();

    let Some(missing_id) = constraints::param::is_valid_old_style(&names_id, declared_names).collect(resolver, span)
    else {
        return Vec::new();
    };
    let ty = QualifiedType::plain(resolver.sema.builtins.int);
    missing_id
        .into_iter()
        .map(|string_id| Name::new(string_id, Span::default()))
        .map(|name| Symbol::param(name, ty, Storage::Auto))
        .for_each(|sym| {
            let _ = resolver.declare(sym, &Span::default());
        });
    names.iter().filter_map(|name| resolver.lookup_ordinary(name.id)).collect()
}

fn add_param_declarator(
    resolver: &mut Resolver,
    specifiers: &[DeclarationSpecifier],
    decl: &DeclaratorNode,
    span: &Span,
) -> Option<SymbolId> {
    let qualif = base_type(resolver, specifiers, span);
    let (ty, decl) = declared_type(resolver, qualif, decl)?;
    let ty = resolver.sema.types.adjust_param(ty);
    let declared_storage = constraints::specifier::storage_of(specifiers).collect(resolver, span);
    if let Some(storage) = declared_storage {
        constraints::param::check_param_storage(storage).collect(resolver, span)?;
    }
    let name = decl.name()?;
    if !ty.is_void(resolver.sema) {
        constraints::param::check_complete_param(ty.is_complete(resolver.sema), ty).collect(resolver, &decl.span);
    }
    let storage = declared_storage.unwrap_or(Storage::Auto);
    let sym = Symbol::param(name, ty, storage);
    Some(resolver.declare(sym, &decl.span))
}

pub fn implicit_declare_function(resolver: &mut Resolver, fn_name: &Name, span: &Span) {
    let ret = QualifiedType::plain(resolver.sema.builtins.int);
    let fn_ty = resolver.sema.types.function(ret, ParamTypes::Unspecified);
    let ty = QualifiedType::plain(fn_ty);
    let prior = resolver.sema.linkage_of_name(fn_name.id);
    let mut sym = Symbol::function(*fn_name, ty, Storage::Extern);
    sym.linkage = Symbol::linkage_of(resolver.scope_kind(), Some(Storage::Extern), SymbolKind::Function, prior);
    sym.definition = DefinitionState::Declared;
    resolver.declare(sym, span);
}
