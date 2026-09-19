use crate::ast::UnaryOp;
use crate::semantic::diagnostic::{Diag, Diagnostic};
use crate::semantic::{QualifiedType, Sema, ValueCategory};

pub fn check_assignable(sema: &Sema, kind: ValueCategory, ty: QualifiedType) -> Diag<()> {
    if kind == ValueCategory::RValue {
        return Diag::err((), Diagnostic::AssignToRValue);
    }
    if ty.is_const {
        return Diag::err((), Diagnostic::ConstAssignment(ty));
    }
    if ty.has_const_member(sema) {
        return Diag::err((), Diagnostic::ConstMemberAssignment(ty));
    }
    Diag::ok(())
}

pub fn check_address_of(
    kind: ValueCategory,
    is_function: bool,
    is_register: bool,
    is_bit_field: bool,
    ty: QualifiedType,
) -> Diag<()> {
    if kind == ValueCategory::RValue && !is_function {
        return Diag::err((), Diagnostic::RValueAddress(ty));
    }
    if is_register {
        return Diag::err((), Diagnostic::RegisterAddress);
    }
    if is_bit_field {
        return Diag::err((), Diagnostic::BitFieldAddress);
    }
    Diag::ok(())
}

pub fn check_sizeof(
    is_bit_field: bool,
    is_void: bool,
    is_function: bool,
    is_complete: bool,
    ty: QualifiedType,
) -> Diag<()> {
    if is_bit_field {
        return Diag::err((), Diagnostic::SizeofBitfield);
    }
    if is_void {
        return Diag::err((), Diagnostic::SizeofVoid);
    }
    if is_function {
        return Diag::err((), Diagnostic::SizeofFunction);
    }
    if !is_complete {
        return Diag::err((), Diagnostic::SizeofIncomplete(ty));
    }
    Diag::ok(())
}

pub fn check_inc_dec(
    op: UnaryOp,
    is_scalar: bool,
    non_object_pointee: Option<(QualifiedType, bool)>,
    ty: QualifiedType,
) -> Diag<()> {
    if let Some((pointee, is_function)) = non_object_pointee {
        return match is_function {
            true => Diag::err((), Diagnostic::BadPostIncDec(op, ty)),
            false => Diag::err((), Diagnostic::IncompleteType(pointee)),
        };
    }
    if !is_scalar {
        return Diag::err((), Diagnostic::BadPostIncDec(op, ty));
    }
    Diag::ok(())
}
