use crate::ast::{DeclarationSpecifier, Qualifier, Storage, TypeSpecifier, Value};
use crate::semantic::diagnosis::{Diag, Diagnosis};
use crate::semantic::{QualifiedType, ResolvedType, ScopeKind};

/// 6.5.1 Storage-class specifiers
/// At most, one storage-class specifier may be given in the declaration specifiers in a declaration
pub fn get_storage(specifiers: &[DeclarationSpecifier]) -> Diag<Option<Storage>> {
    let mut storages = specifiers.iter().filter_map(|s| match s {
        DeclarationSpecifier::Storage(s) => Some(s),
        _ => None,
    });
    let ret = storages.next().cloned();
    if storages.next().is_some() {
        Diag::err(ret, Diagnosis::MultipleStorageSpecifiers)
    } else {
        Diag::ok(ret)
    }
}

/// 6.5.1 Storage-class specifiers
/// The declaration of an identifier for a function that has block scope shall have no explicit storage-class specifier other than extern.
pub fn extern_function_only(scope_type: ScopeKind, storage: Storage) -> Diag<()> {
    if matches!(scope_type, ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern {
        Diag::err((), Diagnosis::BlockScopeNotExtern)
    } else {
        Diag::ok(())
    }
}

/// 6.5.2 Type specifiers
/// Each list of type specifiers shall be one of the following sets; the type specifiers
/// may occur in any order. "int, signed, signed int, or no type specifiers" is the empty set here.
#[rustfmt::skip]
const BASIC_TYPES: &[(&[TypeSpecifier], ResolvedType)] = {
    use ResolvedType as R;
    use TypeSpecifier::{Char, Double, Float, Int, Long, Short, Signed, Unsigned, Void};
    &[
        (&[],                     R::Int),
        (&[Void],                 R::Void),
        (&[Char],                 R::Char),
        (&[Signed, Char],         R::SignedChar),
        (&[Unsigned, Char],       R::UnsignedChar),
        (&[Short],                R::Short),
        (&[Signed, Short],        R::Short),
        (&[Short, Int],           R::Short),
        (&[Signed, Short, Int],   R::Short),
        (&[Unsigned, Short],      R::UnsignedShort),
        (&[Unsigned, Short, Int], R::UnsignedShort),
        (&[Int],                  R::Int),
        (&[Signed],               R::Int),
        (&[Signed, Int],          R::Int),
        (&[Unsigned],             R::UnsignedInt),
        (&[Unsigned, Int],        R::UnsignedInt),
        (&[Long],                 R::Long),
        (&[Signed, Long],         R::Long),
        (&[Long, Int],            R::Long),
        (&[Signed, Long, Int],    R::Long),
        (&[Unsigned, Long],       R::UnsignedLong),
        (&[Unsigned, Long, Int],  R::UnsignedLong),
        (&[Float],                R::Float),
        (&[Double],               R::Double),
        (&[Long, Double],         R::LongDouble),
    ]
};

fn same_set(types: &[&TypeSpecifier], set: &[TypeSpecifier]) -> bool {
    types.len() == set.len() && set.iter().all(|s| types.contains(&s))
}

pub fn basic_type(types: &[&TypeSpecifier]) -> Diag<Option<ResolvedType>> {
    match BASIC_TYPES.iter().find(|(set, _)| same_set(types, set)) {
        Some((_, ty)) => Diag::ok(Some(ty.clone())),
        None => Diag::err(None, Diagnosis::InvalidTypeSpecifier),
    }
}

/// 6.5.2.1 Structure and union specifiers
/// A bit-field is declared with a type other than int, signed int, or unsigned int (6.5.2.1).
pub fn check_bit_width(ty: &ResolvedType, value: Option<Value>) -> Diag<Option<i32>> {
    if !matches!(ty, ResolvedType::Int | ResolvedType::UnsignedInt) {
        return Diag::err(None, Diagnosis::NonIntBitFieldType);
    };
    let Some(value) = value else { return Diag::ok(None) };
    let Some(int_value) = value.get_integer_value() else {
        return Diag::err(None, Diagnosis::NonIntegerConstantExpression);
    };

    Diag::ok(Some(int_value as i32))
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
        return Diag::err(ret, Diagnosis::DuplicateTypeQualifiers);
    }
    Diag::ok(ret)
}

pub fn get_qualifier(specifiers: &[DeclarationSpecifier]) -> Diag<(bool, bool)> {
    let a = specifiers.iter().filter_map(|s| match s {
        &DeclarationSpecifier::Qualifier(q) => Some(q),
        _ => None,
    });
    check_qualifier(a)
}

/// 6.5.4.3 Function declarators (including prototypes)
/// A function declarator shall not specify a return type that is a function type or an array type.
pub fn check_return_type(ret: &ResolvedType, ty: QualifiedType) -> Diag<()> {
    match ret {
        ResolvedType::Array { .. } => Diag::err((), Diagnosis::FunctionReturningArray(ty)),
        ResolvedType::Function { .. } => Diag::err((), Diagnosis::FunctionReturningFunction(ty)),
        _ => Diag::ok(()),
    }
}
