use crate::ast::{DeclarationSpecifier, Qualifier, Storage, TypeSpecifier};
use crate::semantic::diagnosis::{Diag, DiagnosisInner};
use crate::semantic::{ResolvedType, ScopeType, TypeSpecifierCounter};

/// 6.5.1 Storage-class specifiers
/// At most, one storage-class specifier may be given in the declaration specifiers in a declaration
pub fn get_storage(specifiers: &[DeclarationSpecifier]) -> Diag<Option<Storage>> {
    let mut storages = specifiers.iter().filter_map(|s| match s {
        DeclarationSpecifier::Storage(s) => Some(s),
        _ => None,
    });
    let ret = storages.next().cloned();
    if storages.next().is_some() {
        Diag::with_diag(ret, DiagnosisInner::MultipleStorageSpecifiers)
    } else {
        Diag::res(ret)
    }
}

// 6.5.1 Storage-class specifiers
// The declaration of an identifier for a function that has block scope shall have no explicit storage-class specifier other than extern.
pub fn extern_function_only(scope_type: ScopeType, storage: Storage) -> Diag<()> {
    if scope_type == ScopeType::Block && storage != Storage::Extern {
        Diag::with_diag((), DiagnosisInner::BlockScopeNotExtern)
    } else {
        Diag::res(())
    }
}

/// 6.5.2 Type specifiers
/// Each list of type specifiers shall be one of the following sets...
#[rustfmt::skip]
pub fn resolve_type(specifiers: &[DeclarationSpecifier]) -> Diag<Option<ResolvedType>> {
    let types: Vec<_> = specifiers
        .iter()
        .filter_map(|s| match s {
            DeclarationSpecifier::Type(t) => Some(t),
            _ => None,
        })
        .collect();

    match types.as_slice() {
        [] => return Diag::res_some(ResolvedType::Int),
        [TypeSpecifier::Struct(t)] => return Diag::res_some(ResolvedType::Struct(*t)),
        [TypeSpecifier::Union(t)] => return Diag::res_some(ResolvedType::Union(*t)),
        [TypeSpecifier::Enum(t)] => return Diag::res_some(ResolvedType::Enum(*t)),
        [TypeSpecifier::TypedefName(t)] => return Diag::res_some(ResolvedType::Typedef(*t)),
        _ => ()
    }
    let Some(a) = TypeSpecifierCounter::count(types.as_slice()) else {
        return Diag::with_diag(None, DiagnosisInner::InvalidTypeSpecifer);
    };
    //   s, u, v, c, s, i, l, f, d
    match  a {
        [0, 0, 1, 0, 0, 0, 0, 0, 0] => Diag::res_some(ResolvedType::Void),
        [0, 0, 0, 1, 0, 0, 0, 0, 0] => Diag::res_some(ResolvedType::Char),
        [1, 0, 0, 1, 0, 0, 0, 0, 0] => Diag::res_some(ResolvedType::SignedChar),
        [0, 1, 0, 1, 0, 0, 0, 0, 0] => Diag::res_some(ResolvedType::UnsignedChar),
        [0, 0, 0, 0, 1, 0, 0, 0, 0] => Diag::res_some(ResolvedType::Short),
        [0, 0, 0, 0, 1, 1, 0, 0, 0] => Diag::res_some(ResolvedType::Short),
        [1, 0, 0, 0, 1, 0, 0, 0, 0] => Diag::res_some(ResolvedType::Short),
        [1, 0, 0, 0, 1, 1, 0, 0, 0] => Diag::res_some(ResolvedType::Short),
        [0, 1, 0, 0, 1, 0, 0, 0, 0] => Diag::res_some(ResolvedType::UnsignedShort),
        [0, 1, 0, 0, 1, 1, 0, 0, 0] => Diag::res_some(ResolvedType::UnsignedShort),
        [0, 0, 0, 0, 0, 1, 0, 0, 0] => Diag::res_some(ResolvedType::Int),
        [1, 0, 0, 0, 0, 0, 0, 0, 0] => Diag::res_some(ResolvedType::Int),
        [1, 0, 0, 0, 0, 1, 0, 0, 0] => Diag::res_some(ResolvedType::Int),
        [0, 0, 0, 0, 0, 0, 1, 0, 0] => Diag::res_some(ResolvedType::Long),
        [0, 0, 0, 0, 0, 1, 1, 0, 0] => Diag::res_some(ResolvedType::Long),
        [1, 0, 0, 0, 0, 0, 1, 0, 0] => Diag::res_some(ResolvedType::Long),
        [1, 0, 0, 0, 0, 1, 1, 0, 0] => Diag::res_some(ResolvedType::Long),
        [0, 1, 0, 0, 0, 0, 1, 0, 0] => Diag::res_some(ResolvedType::UnsignedLong),
        [0, 1, 0, 0, 0, 1, 1, 0, 0] => Diag::res_some(ResolvedType::UnsignedLong),
        [0, 0, 0, 0, 0, 0, 0, 1, 0] => Diag::res_some(ResolvedType::Float),
        [0, 0, 0, 0, 0, 0, 0, 0, 1] => Diag::res_some(ResolvedType::Double),
        [0, 0, 0, 0, 0, 0, 1, 0, 1] => Diag::res_some(ResolvedType::LongDouble),
        _ => Diag::with_diag(None, DiagnosisInner::InvalidTypeSpecifer),
    }
}

/// 6.5.3 Type qualifiers
/// The same type qualifier shall not appear more than once in the same specifier list or qualifier list, either directly or via one or more typedefs.
pub fn get_qualifier(specifiers: &[DeclarationSpecifier]) -> Diag<(bool, bool)> {
    let const_count = specifiers
        .iter()
        .filter(|q| matches!(q, DeclarationSpecifier::Qualifier(Qualifier::Const)))
        .count();
    let volatile_count = specifiers
        .iter()
        .filter(|q| matches!(q, DeclarationSpecifier::Qualifier(Qualifier::Volatile)))
        .count();

    let ret = (const_count > 1, volatile_count > 1);

    if const_count > 1 || volatile_count > 1 {
        return Diag::with_diag(ret, DiagnosisInner::DuplicateTypeQualifers);
    }
    Diag::res(ret)
}
