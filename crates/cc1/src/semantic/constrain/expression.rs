use crate::ast::UnaryOp;
use crate::semantic::diagnosis::{Diag, Diagnosis};
use crate::semantic::{ExpressionKind, QualifiedType, Sema};

pub fn check_assignable(sema: &Sema, kind: ExpressionKind, ty: QualifiedType) -> Diag<()> {
    if kind == ExpressionKind::RValue {
        return Diag::err((), Diagnosis::AssignToRValue);
    }
    if ty.is_const {
        return Diag::err((), Diagnosis::ConstAssignment(ty));
    }
    if ty.has_const_member(sema) {
        return Diag::err((), Diagnosis::ConstMemberAssignment(ty));
    }
    Diag::ok(())
}

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
