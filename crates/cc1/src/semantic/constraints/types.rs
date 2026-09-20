use crate::ast::{ConstValue, Name};
use crate::semantic::diagnostic::{Diag, Diagnostic};
use crate::semantic::{DeclaredParams, QualifiedType, ResolvedType};

/// 6.5.2.3 A type specifier of the form `enum identifier` without an enumerator list shall only
/// appear after the type it specifies is complete.
pub fn check_enum_reference(is_complete: bool, name: Option<Name>) -> Diag<()> {
    match is_complete {
        true => Diag::ok(()),
        false => Diag::err((), Diagnostic::ForwardEnumReference(name)),
    }
}

pub fn check_return_type(ret: &ResolvedType, ty: QualifiedType) -> Diag<()> {
    match ret {
        ResolvedType::Array { .. } => Diag::err((), Diagnostic::FunctionReturningArray(ty)),
        ResolvedType::Function { .. } => Diag::err((), Diagnostic::FunctionReturningFunction(ty)),
        _ => Diag::ok(()),
    }
}

pub fn check_definition_return(is_valid: bool, ty: QualifiedType) -> Diag<()> {
    match is_valid {
        true => Diag::ok(()),
        false => Diag::err((), Diagnostic::IncompleteReturn(ty)),
    }
}

pub fn check_complete_object(is_complete: bool, ty: QualifiedType) -> Diag<()> {
    match is_complete {
        true => Diag::ok(()),
        false => Diag::err((), Diagnostic::IncompleteVariable(ty)),
    }
}

pub fn check_member_type(is_object: bool, ty: QualifiedType) -> Diag<()> {
    match is_object {
        true => Diag::ok(()),
        false => Diag::err((), Diagnostic::InvalidMemberType(ty)),
    }
}

pub fn check_element_type(is_object: bool, ty: QualifiedType) -> Diag<()> {
    match is_object {
        true => Diag::ok(()),
        false => Diag::err((), Diagnostic::InvalidElementType(ty)),
    }
}

pub fn check_bit_width(
    ty: &ResolvedType,
    value: Option<ConstValue>,
    name: Option<Name>,
) -> Diag<Option<i32>> {
    if !matches!(ty, ResolvedType::Int | ResolvedType::UnsignedInt) {
        return Diag::err(None, Diagnostic::NonIntBitFieldType);
    };
    let Some(value) = value else { return Diag::ok(None) };
    let Some(width) = value.get_integer_value() else {
        return Diag::err(None, Diagnostic::NonIntegerConstantExpression);
    };
    if value.is_negative() {
        return Diag::err(None, Diagnostic::NegativeBitFieldWidth(name, value.to_i64()));
    }
    let Some(bits) = ty.bits() else {
        return Diag::err(None, Diagnostic::NonIntBitFieldType);
    };
    if width > u64::from(bits) {
        return Diag::err(None, Diagnostic::BitFieldWidthTooLarge(name, width, bits));
    }
    match (width, name) {
        (0, Some(name)) => Diag::err(None, Diagnostic::ZeroWidthNamedBitField(name)),
        _ => Diag::ok(Some(width as i32)),
    }
}

pub fn extract_function_declarator(params: Option<DeclaredParams>) -> Diag<Option<DeclaredParams>> {
    params.map_or_else(|| Diag::err(None, Diagnostic::NotFunctionTypeDeclarator), |params| Diag::ok(Some(params)))
}
