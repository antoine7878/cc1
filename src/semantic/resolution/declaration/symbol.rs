use crate::arena::ResolveWith;
use crate::ast::{DeclarationNode, DeclarationSpecifier, DeclaratorNode, InitDeclaratorNode, Storage, TypeSpecifier};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, QualifiedType, ResolvedType, ScopeKind, Symbol, SymbolId, SymbolKind,
    SymbolResolver, constrain,
};

pub fn requires_complete_object(resolver: &SymbolResolver, ty: QualifiedType, storage: Storage, is_init: bool) -> bool {
    if ty.is_void(resolver.sema) {
        return true;
    }
    let is_unsized_array = matches!(ty.id.resolve(resolver.sema), ResolvedType::Array { len: None, .. });
    if is_init && is_unsized_array {
        return false;
    }
    if is_init {
        return true;
    }
    matches!(resolver.scope_kind(), ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern
}

pub fn check_declaration(resolver: &mut SymbolResolver, ctx: &Context, node: &DeclarationNode) {
    let specifiers = &node.specifiers;
    let span = &node.span;
    if resolver.scope_kind() == ScopeKind::File {
        constrain::specifier::check_external_specifiers(specifiers).collect(resolver, span);
    }
    if node.init_declarators.is_empty() && !declares_tag(ctx, specifiers) {
        resolver.add_diag(Diag::err((), Diagnosis::EmptyDeclaration), span);
    }
}

pub fn declared_type_watched(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    qualif: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode, bool)> {
    let before = resolver.sema.diagnosis.len();
    let (ty, core) = declared_type(resolver, ctx, qualif, decl)?;
    Some((ty, core, resolver.sema.diagnosis.len() != before))
}

pub fn classify(
    resolver: &mut SymbolResolver,
    ty: QualifiedType,
    declared_storage: Option<Storage>,
    span: &Span,
) -> (Storage, SymbolKind) {
    let is_function = matches!(ty.id.resolve(resolver.sema), ResolvedType::Function { .. });
    if let Some(declared_storage) = declared_storage
        && declared_storage != Storage::Typedef
        && is_function
    {
        constrain::specifier::extern_function_only(resolver.scope_kind(), declared_storage).collect(resolver, span);
    }
    let default_storage = if is_function { Storage::Extern } else { Storage::Auto };
    let storage = declared_storage.unwrap_or(default_storage);
    let kind = match storage {
        Storage::Typedef => SymbolKind::Typedef,
        _ if is_function => SymbolKind::Function,
        _ => SymbolKind::Variable,
    };
    (storage, kind)
}

pub fn declare_symbol(
    resolver: &mut SymbolResolver,
    mut sym: Symbol,
    declared_storage: Option<Storage>,
    span: &Span,
) -> SymbolId {
    let scope_kind = resolver.scope_kind();
    let prior = resolver.sema.linkage_of_name(sym.name.id);
    sym.linkage = Symbol::linkage_of(scope_kind, declared_storage, sym.kind, prior);
    sym.duration = Symbol::duration_of(scope_kind, declared_storage, sym.kind);
    sym.definition = Symbol::definition_of(scope_kind, declared_storage, sym.is_init, sym.kind);
    resolver.declare(sym, span)
}

pub fn declare_init_declarator(
    resolver: &mut SymbolResolver,
    ctx: &Context,
    init_declarator: &InitDeclaratorNode,
    qualif: Option<QualifiedType>,
    declared_storage: Option<Storage>,
) -> Option<()> {
    let (ty, core, already_diagnosed) = declared_type_watched(resolver, ctx, qualif, &init_declarator.declarator)?;
    let name = core.ident(ctx)?;
    let is_init = init_declarator.initializer.is_some();
    let (storage, kind) = classify(resolver, ty, declared_storage, &core.span);
    if kind == SymbolKind::Variable && !already_diagnosed && requires_complete_object(resolver, ty, storage, is_init) {
        constrain::ty::check_complete_object(ty.is_complete(resolver.sema), ty).collect(resolver, &core.span);
    }
    let sym = Symbol::new(name, Some(ty), Some(storage), kind, is_init);
    let sym_id = declare_symbol(resolver, sym, declared_storage, &core.span);
    resolver.sema.declarations.insert(init_declarator.declarator.id, sym_id);
    Some(())
}

pub fn declares_tag(ctx: &Context, specifiers: &[DeclarationSpecifier]) -> bool {
    specifiers.iter().any(|specifier| match specifier {
        DeclarationSpecifier::Type(TypeSpecifier::Struct(id)) => id.resolve(ctx).name.is_some(),
        DeclarationSpecifier::Type(TypeSpecifier::Union(id)) => id.resolve(ctx).name.is_some(),
        DeclarationSpecifier::Type(TypeSpecifier::Enum(_)) => true,
        _ => false,
    })
}
