use crate::ast::{DeclarationSpecifier, Qualifier, Storage, TypeSpecifier, Value};
use crate::semantic::diagnosis::{Diag, Diagnosis};
use crate::semantic::{ResolvedType, ScopeKind, TypeSpecifierCounter};

/// 6.5.1 Storage-class specifiers
/// At most, one storage-class specifier may be given in the declaration specifiers in a declaration
pub fn get_storage(specifiers: &[DeclarationSpecifier]) -> Diag<Option<Storage>> {
    let mut storages = specifiers.iter().filter_map(|s| match s {
        DeclarationSpecifier::Storage(s) => Some(s),
        _ => None,
    });
    let ret = storages.next().cloned();
    if storages.next().is_some() {
        Diag::with_diag(ret, Diagnosis::MultipleStorageSpecifiers)
    } else {
        Diag::res(ret)
    }
}

/// 6.5.1 Storage-class specifiers
/// The declaration of an identifier for a function that has block scope shall have no explicit storage-class specifier other than extern.
pub fn extern_function_only(scope_type: ScopeKind, storage: Storage) -> Diag<()> {
    if matches!(scope_type, ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern {
        Diag::with_diag((), Diagnosis::BlockScopeNotExtern)
    } else {
        Diag::res(())
    }
}

#[rustfmt::skip]
pub fn basic_type(types: &[&TypeSpecifier]) -> Diag<Option<ResolvedType>> {
    if types.is_empty() {
        return Diag::some(ResolvedType::Int);
    }
    let Some(a) = TypeSpecifierCounter::count(types) else {
        return Diag::with_diag(None, Diagnosis::InvalidTypeSpecifer);
    };
    //   s, u, v, c, s, i, l, f, d
    match  a {
        [0, 0, 1, 0, 0, 0, 0, 0, 0] => Diag::some(ResolvedType::Void),
        [0, 0, 0, 1, 0, 0, 0, 0, 0] => Diag::some(ResolvedType::Char),
        [1, 0, 0, 1, 0, 0, 0, 0, 0] => Diag::some(ResolvedType::SignedChar),
        [0, 1, 0, 1, 0, 0, 0, 0, 0] => Diag::some(ResolvedType::UnsignedChar),
        [0, 0, 0, 0, 1, 0, 0, 0, 0] => Diag::some(ResolvedType::Short),
        [0, 0, 0, 0, 1, 1, 0, 0, 0] => Diag::some(ResolvedType::Short),
        [1, 0, 0, 0, 1, 0, 0, 0, 0] => Diag::some(ResolvedType::Short),
        [1, 0, 0, 0, 1, 1, 0, 0, 0] => Diag::some(ResolvedType::Short),
        [0, 1, 0, 0, 1, 0, 0, 0, 0] => Diag::some(ResolvedType::UnsignedShort),
        [0, 1, 0, 0, 1, 1, 0, 0, 0] => Diag::some(ResolvedType::UnsignedShort),
        [0, 0, 0, 0, 0, 1, 0, 0, 0] => Diag::some(ResolvedType::Int),
        [1, 0, 0, 0, 0, 0, 0, 0, 0] => Diag::some(ResolvedType::Int),
        [1, 0, 0, 0, 0, 1, 0, 0, 0] => Diag::some(ResolvedType::Int),
        [0, 1, 0, 0, 0, 0, 0, 0, 0] => Diag::some(ResolvedType::UnsignedInt),
        [0, 1, 0, 0, 0, 1, 0, 0, 0] => Diag::some(ResolvedType::UnsignedInt),
        [0, 0, 0, 0, 0, 0, 1, 0, 0] => Diag::some(ResolvedType::Long),
        [0, 0, 0, 0, 0, 1, 1, 0, 0] => Diag::some(ResolvedType::Long),
        [1, 0, 0, 0, 0, 0, 1, 0, 0] => Diag::some(ResolvedType::Long),
        [1, 0, 0, 0, 0, 1, 1, 0, 0] => Diag::some(ResolvedType::Long),
        [0, 1, 0, 0, 0, 0, 1, 0, 0] => Diag::some(ResolvedType::UnsignedLong),
        [0, 1, 0, 0, 0, 1, 1, 0, 0] => Diag::some(ResolvedType::UnsignedLong),
        [0, 0, 0, 0, 0, 0, 0, 1, 0] => Diag::some(ResolvedType::Float),
        [0, 0, 0, 0, 0, 0, 0, 0, 1] => Diag::some(ResolvedType::Double),
        [0, 0, 0, 0, 0, 0, 1, 0, 1] => Diag::some(ResolvedType::LongDouble),
        _ => Diag::with_diag(None, Diagnosis::InvalidTypeSpecifer),
    }
}

/// 6.5.2.1 Structure and union specifiers
/// A bit-field is declared with a type other than int, signed int, or unsigned int (6.5.2.1).
pub fn check_bit_width(ty: &ResolvedType, value: Option<Value>) -> Diag<Option<i32>> {
    if !matches!(ty, ResolvedType::Int | ResolvedType::UnsignedInt) {
        return Diag::none_diag(Diagnosis::NonIntBitFieldType);
    };
    let Some(value) = value else { return Diag::none() };
    let Some(int_value) = value.get_integer_value() else {
        return Diag::none_diag(Diagnosis::NonIntegerConstantExpression);
    };

    Diag::some(int_value as i32)
}

/// 6.5.3 Type qualifiers
/// The same type qualifier shall not appear more than once in the same specifier list or qualifier list, either directly or via one or more typedefs.
pub fn check_qualifier<I>(qualifiers: I) -> Diag<(bool, bool)>
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
        return Diag::with_diag(ret, Diagnosis::DuplicateTypeQualifers);
    }
    Diag::res(ret)
}

pub fn get_qualifier(specifiers: &[DeclarationSpecifier]) -> Diag<(bool, bool)> {
    let a = specifiers.iter().filter_map(|s| match s {
        &DeclarationSpecifier::Qualifier(q) => Some(q),
        _ => None,
    });
    check_qualifier(a)
}
