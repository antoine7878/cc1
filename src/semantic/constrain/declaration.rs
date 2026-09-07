use crate::ast::{DeclarationSpecifier, Name, Qualifier, Storage, TypeSpecifier, Value};
use crate::semantic::diagnosis::{Diag, Diagnosis};
use crate::semantic::{QualifiedType, ResolvedType, ScopeKind};
use crate::target::Target;

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

pub fn extern_function_only(scope_type: ScopeKind, storage: Storage) -> Diag<()> {
    if matches!(scope_type, ScopeKind::Block | ScopeKind::Function) && storage != Storage::Extern {
        Diag::err((), Diagnosis::BlockScopeNotExtern)
    } else {
        Diag::ok(())
    }
}

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

pub fn check_bit_width(
    target: &Target,
    ty: &ResolvedType,
    value: Option<Value>,
    name: Option<Name>,
) -> Diag<Option<i32>> {
    if !matches!(ty, ResolvedType::Int | ResolvedType::UnsignedInt) {
        return Diag::err(None, Diagnosis::NonIntBitFieldType);
    };
    let Some(value) = value else { return Diag::ok(None) };
    let Some(width) = value.get_integer_value() else {
        return Diag::err(None, Diagnosis::NonIntegerConstantExpression);
    };
    if value.is_negative() {
        return Diag::err(None, Diagnosis::NegativeBitFieldWidth(name, value.to_i64()));
    }
    let Some(bits) = target.bits(ty) else {
        return Diag::err(None, Diagnosis::NonIntBitFieldType);
    };
    if width > u64::from(bits) {
        return Diag::err(None, Diagnosis::BitFieldWidthTooLarge(name, width, bits));
    }
    match (width, name) {
        (0, Some(name)) => Diag::err(None, Diagnosis::ZeroWidthNamedBitField(name)),
        _ => Diag::ok(Some(width as i32)),
    }
}

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

pub fn check_return_type(ret: &ResolvedType, ty: QualifiedType) -> Diag<()> {
    match ret {
        ResolvedType::Array { .. } => Diag::err((), Diagnosis::FunctionReturningArray(ty)),
        ResolvedType::Function { .. } => Diag::err((), Diagnosis::FunctionReturningFunction(ty)),
        _ => Diag::ok(()),
    }
}

pub fn check_complete_object(is_complete: bool, ty: QualifiedType) -> Diag<()> {
    match is_complete {
        true => Diag::ok(()),
        false => Diag::err((), Diagnosis::IncompleteVariable(ty)),
    }
}

pub fn check_member_type(is_object: bool, ty: QualifiedType) -> Diag<()> {
    match is_object {
        true => Diag::ok(()),
        false => Diag::err((), Diagnosis::InvalidMemberType(ty)),
    }
}

pub fn check_element_type(is_object: bool, ty: QualifiedType) -> Diag<()> {
    match is_object {
        true => Diag::ok(()),
        false => Diag::err((), Diagnosis::InvalidElementType(ty)),
    }
}
