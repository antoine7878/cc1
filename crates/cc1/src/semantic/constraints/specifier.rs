use crate::ast::{DeclarationSpecifier, InitDeclaratorNode, Qualifier, Storage, TypeSpecifier};
use crate::semantic::diagnostic::{Diag, Diagnostic};
use crate::semantic::{Linkage, ResolvedType, ScopeKind};

pub fn storage_of(specifiers: &[DeclarationSpecifier]) -> Diag<Option<Storage>> {
    let mut storages = specifiers.iter().filter_map(|s| match s {
        DeclarationSpecifier::Storage(s) => Some(s),
        _ => None,
    });
    let ret = storages.next().cloned();
    if storages.next().is_some() { Diag::err(ret, Diagnostic::MultipleStorageSpecifiers) } else { Diag::ok(ret) }
}

pub fn qualifiers_of(specifiers: &[DeclarationSpecifier]) -> Diag<(bool, bool)> {
    let a = specifiers.iter().filter_map(|s| match s {
        &DeclarationSpecifier::Qualifier(q) => Some(q),
        _ => None,
    });
    check_qualifiers(a)
}

pub fn check_qualifiers<I>(qualifiers: I) -> Diag<(bool, bool)>
where
    I: IntoIterator<Item = Qualifier>,
{
    let mut const_count = 0;
    let mut volatile_count = 0;
    for qualifier in qualifiers {
        match qualifier {
            Qualifier::Const => const_count += 1,
            Qualifier::Volatile => volatile_count += 1,
        }
    }
    let ret = (const_count >= 1, volatile_count >= 1);
    if const_count > 1 || volatile_count > 1 {
        return Diag::err(ret, Diagnostic::DuplicateTypeQualifiers);
    }
    Diag::ok(ret)
}

const BASIC_TYPES: &[(&[TypeSpecifier], ResolvedType)] = {
    use ResolvedType as R;
    use TypeSpecifier::{Char, Double, Float, Int, Long, Short, Signed, Unsigned, Void};
    &[
        (&[], R::Int),
        (&[Void], R::Void),
        (&[Char], R::Char),
        (&[Signed, Char], R::SignedChar),
        (&[Unsigned, Char], R::UnsignedChar),
        (&[Short], R::Short),
        (&[Signed, Short], R::Short),
        (&[Short, Int], R::Short),
        (&[Signed, Short, Int], R::Short),
        (&[Unsigned, Short], R::UnsignedShort),
        (&[Unsigned, Short, Int], R::UnsignedShort),
        (&[Int], R::Int),
        (&[Signed], R::Int),
        (&[Signed, Int], R::Int),
        (&[Unsigned], R::UnsignedInt),
        (&[Unsigned, Int], R::UnsignedInt),
        (&[Long], R::Long),
        (&[Signed, Long], R::Long),
        (&[Long, Int], R::Long),
        (&[Signed, Long, Int], R::Long),
        (&[Unsigned, Long], R::UnsignedLong),
        (&[Unsigned, Long, Int], R::UnsignedLong),
        (&[Float], R::Float),
        (&[Double], R::Double),
        (&[Long, Double], R::LongDouble),
    ]
};

fn same_set(types: &[&TypeSpecifier], set: &[TypeSpecifier]) -> bool {
    types.len() == set.len() && set.iter().all(|s| types.contains(&s))
}

pub fn basic_type(types: &[&TypeSpecifier]) -> Diag<Option<ResolvedType>> {
    match BASIC_TYPES.iter().find(|(set, _)| same_set(types, set)) {
        Some((_, ty)) => Diag::ok(Some(ty.clone())),
        None => Diag::err(None, Diagnostic::InvalidTypeSpecifier),
    }
}

pub fn check_external_specifiers(specifiers: &[DeclarationSpecifier]) -> Diag<()> {
    if specifiers.iter().any(|s| matches!(s, DeclarationSpecifier::Storage(Storage::Auto | Storage::Register))) {
        return Diag::err((), Diagnostic::AutoRegisterExternal);
    }
    Diag::ok(())
}

pub fn check_block_extern_function(scope_type: ScopeKind, storage: Storage) -> Diag<()> {
    if matches!(scope_type, ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern {
        Diag::err((), Diagnostic::BlockScopeNotExtern)
    } else {
        Diag::ok(())
    }
}

pub fn check_block_scope_initializer(scope: ScopeKind, linkage: Linkage, has_initializer: bool) -> Diag<()> {
    let block = matches!(scope, ScopeKind::Block | ScopeKind::Function);
    match block && linkage != Linkage::None && has_initializer {
        true => Diag::err((), Diagnostic::BlockScopeLinkageInitializer),
        false => Diag::ok(()),
    }
}

pub fn check_function_storage(storage: Storage) -> Diag<()> {
    match storage {
        Storage::Static | Storage::Extern => Diag::ok(()),
        _ => Diag::err((), Diagnostic::FunctionAutoExtern),
    }
}

pub fn is_tentative_definition(init_declarator: &InitDeclaratorNode, storage: Option<Storage>) -> bool {
    init_declarator.initializer.is_none() && matches!(storage, None | Some(Storage::Static))
}
