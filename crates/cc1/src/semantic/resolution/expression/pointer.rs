use crate::arena::ResolveWith;
use crate::semantic::{QualifiedType, ResolvedExpression, ResolvedType, Sema, cast};

pub enum PointerMatch {
    Converted,
    Compatible(QualifiedType, QualifiedType),
}

pub fn reconcile_pointers(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    l_null: bool,
    r_null: bool,
) -> Option<PointerMatch> {
    let l_ty = lhs.casted_ty();
    let r_ty = rhs.casted_ty();
    let l = l_ty.id.resolve(sema);
    let r = r_ty.id.resolve(sema);
    if l.is_pointer() && r_null {
        cast::convert(sema, rhs, l_ty.id, true);
        return Some(PointerMatch::Converted);
    }
    if r.is_pointer() && l_null {
        cast::convert(sema, lhs, r_ty.id, true);
        return Some(PointerMatch::Converted);
    }
    let (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) = (l, r) else {
        return None;
    };
    if i1.is_compatible_ignoring_qualifiers(sema, i2) {
        return Some(PointerMatch::Compatible(*i1, *i2));
    }
    if i1.is_void(sema) && !i2.is_function(sema) {
        cast::convert(sema, rhs, l_ty.id, false);
        return Some(PointerMatch::Converted);
    }
    if i2.is_void(sema) && !i1.is_function(sema) {
        cast::convert(sema, lhs, r_ty.id, false);
        return Some(PointerMatch::Converted);
    }
    None
}

pub fn both_pointers(sema: &Sema, lhs: &ResolvedExpression, rhs: &ResolvedExpression) -> bool {
    lhs.casted_ty().is_pointer(sema) && rhs.casted_ty().is_pointer(sema)
}
