use crate::arena::ResolveWith;
use crate::ast::{DeclarationNode, DeclarationSpecifier, DeclaratorNode, InitDeclaratorNode, Storage, TypeSpecifier};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::resolution::declaration::*;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, QualifiedType, ResolvedType, ScopeKind, Sema, Symbol, SymbolId, SymbolKind,
    constrain,
};

pub fn requires_complete_object(sema: &Sema, ty: QualifiedType, storage: Storage, is_init: bool) -> bool {
    if ty.is_void(sema) {
        return true;
    }
    let is_unsized_array = matches!(ty.id.resolve(sema), ResolvedType::Array { len: None, .. });
    if is_init && is_unsized_array {
        return false;
    }
    if is_init {
        return true;
    }
    matches!(sema.scopes.kind(), ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern
}

pub fn check_declaration(sema: &mut Sema, ctx: &Context, node: &DeclarationNode) {
    let specifiers = &node.specifiers;
    let span = &node.span;
    if sema.scopes.kind() == ScopeKind::File {
        constrain::specifier::check_external_specifiers(specifiers).collect(sema, span);
    }
    if node.init_declarators.is_empty() && !declares_tag(ctx, specifiers) {
        sema.add_diag(Diag::err((), Diagnosis::EmptyDeclaration), span);
    }
}

pub fn declared_type_watched(
    sema: &mut Sema,
    ctx: &Context,
    qualif: Option<QualifiedType>,
    decl: &DeclaratorNode,
) -> Option<(QualifiedType, DeclaratorNode, bool)> {
    let before = sema.diagnosis.len();
    let (ty, core) = declared_type(sema, ctx, qualif, decl)?;
    Some((ty, core, sema.diagnosis.len() != before))
}

pub fn classify(
    sema: &mut Sema,
    ty: QualifiedType,
    declared_storage: Option<Storage>,
    span: &Span,
) -> (Storage, SymbolKind) {
    let is_function = matches!(ty.id.resolve(sema), ResolvedType::Function { .. });
    if let Some(declared_storage) = declared_storage
        && declared_storage != Storage::Typedef
        && is_function
    {
        constrain::specifier::extern_function_only(sema.scopes.kind(), declared_storage).collect(sema, span);
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

pub fn declare_symbol(sema: &mut Sema, mut sym: Symbol, declared_storage: Option<Storage>, span: &Span) -> SymbolId {
    let scope_kind = sema.scopes.kind();
    let prior = sema.linkage_of_name(sym.name.id);
    sym.linkage = Symbol::linkage_of(scope_kind, declared_storage, sym.kind, prior);
    sym.duration = Symbol::duration_of(scope_kind, declared_storage, sym.kind);
    sym.definition = Symbol::definition_of(scope_kind, declared_storage, sym.is_init, sym.kind);
    sema.declare(sym, span)
}

pub fn declare_init_declarator(
    sema: &mut Sema,
    ctx: &Context,
    init_declarator: &InitDeclaratorNode,
    qualif: Option<QualifiedType>,
    declared_storage: Option<Storage>,
) -> Option<()> {
    let (ty, core, already_diagnosed) = declared_type_watched(sema, ctx, qualif, &init_declarator.declarator)?;
    let name = core.ident(ctx)?;
    let is_init = init_declarator.initializer.is_some();
    let (storage, kind) = classify(sema, ty, declared_storage, &core.span);
    if kind == SymbolKind::Variable && !already_diagnosed && requires_complete_object(sema, ty, storage, is_init) {
        constrain::ty::check_complete_object(ty.is_complete(sema), ty).collect(sema, &core.span);
    }
    let sym = Symbol::new(name, Some(ty), Some(storage), kind, is_init);
    let sym_id = declare_symbol(sema, sym, declared_storage, &core.span);
    sema.declarations.insert(init_declarator.declarator.id, sym_id);
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
