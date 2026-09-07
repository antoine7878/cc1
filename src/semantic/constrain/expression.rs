use crate::ast::UnaryOp;
use crate::semantic::diagnosis::{Diag, Diagnosis};
use crate::semantic::{ExpressionKind, QualifiedType};

/// 6.3.16 An assignment operator shall have a modifiable lvalue as its left operand.
pub fn check_assignable(kind: ExpressionKind, ty: QualifiedType) -> Diag<()> {
    if kind == ExpressionKind::RValue {
        return Diag::err((), Diagnosis::AssignToRValue);
    }
    if ty.is_const {
        return Diag::err((), Diagnosis::ConstAssignment(ty));
    }
    Diag::ok(())
}

/// 6.3.3.2 The operand of the unary & operator shall be either a function designator or an lvalue
/// that designates an object that is not a bit-field and is not declared with the register
/// storage-class specifier.
pub fn check_address_of(
    kind: ExpressionKind,
    is_function: bool,
    is_register: bool,
    is_bit_field: bool,
    ty: QualifiedType,
) -> Diag<()> {
    if kind == ExpressionKind::RValue && !is_function {
        return Diag::err((), Diagnosis::RValueAddress(ty));
    }
    if is_register {
        return Diag::err((), Diagnosis::RegisterAddress);
    }
    if is_bit_field {
        return Diag::err((), Diagnosis::BitFieldAddress);
    }
    Diag::ok(())
}

/// 6.3.3.4 The sizeof operator shall not be applied to an expression that has function type or an
/// incomplete type, to the parenthesized name of such a type, or to an lvalue that designates a
/// bit-field object.
pub fn check_sizeof(
    is_bit_field: bool,
    is_void: bool,
    is_function: bool,
    is_complete: bool,
    ty: QualifiedType,
) -> Diag<()> {
    if is_bit_field {
        return Diag::err((), Diagnosis::SizeofBitfield);
    }
    if is_void {
        return Diag::err((), Diagnosis::SizeofVoid);
    }
    if is_function {
        return Diag::err((), Diagnosis::SizeofFunction);
    }
    if !is_complete {
        return Diag::err((), Diagnosis::SizeofIncomplete(ty));
    }
    Diag::ok(())
}

/// 6.3.2.4 and 6.3.3.1 The operand of the increment or decrement operators shall be a modifiable
/// lvalue with qualified or unqualified scalar type. A pointer operand shall point to an object.
pub fn check_inc_dec(
    op: UnaryOp,
    is_scalar: bool,
    non_object_pointee: Option<(QualifiedType, bool)>,
    ty: QualifiedType,
) -> Diag<()> {
    if let Some((pointee, is_function)) = non_object_pointee {
        return match is_function {
            true => Diag::err((), Diagnosis::BadPostIncDec(op, ty)),
            false => Diag::err((), Diagnosis::IncompleteType(pointee)),
        };
    }
    if !is_scalar {
        return Diag::err((), Diagnosis::BadPostIncDec(op, ty));
    }
    Diag::ok(())
}
