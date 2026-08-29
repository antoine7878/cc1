use std::any::Any;
use std::num;

use crate::ast::{self};
use crate::parser::Span;
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, ResolvedTypeId,
    Sema,
};

#[derive(Clone, Copy, Debug)]
pub struct ImplicitCast {
    pub kind: CastKind,
    pub to: QualifiedType,
}
impl ImplicitCast {
    fn new(kind: CastKind, to: QualifiedType) -> Self {
        Self { kind, to }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CastKind {
    IntegerPromotion,   // 6.2.1.1
    IntegerConversion,  // 6.2.1.2
    IntegerToFloating,  // 6.2.1.3
    FloatingConversion, // 6.2.1.4
    FloatingToInteger,  // 6.2.1.4

    FunctionToPointer, // 6.2.2.1
    LValueToRValue,    // 6.2.2.1
    ArrayToPointer,    // 6.2.2.1

    NullPointer,       // 6.2.2.3
    ToVoid,            // 6.3.4
    PointerConversion, // 6.3.16.1
}

// 6.2.2.1 an lvalue that does not have array type is converted to the value stored in the
// designated object (and is no longer an lvalue). If the lvalue has qualified type, the value
// has the unqualified version of the type of the lvalue.
// 6.2.2.1 If the lvalue has an incomplete type and does not have array type, the behavior is undefined.
pub fn lvalue_conversion(sema: &mut Sema, re: &mut ResolvedExpression, span: &Span) {
    function_to_pointer(sema, re);
    array_to_pointer(sema, re);
    l_to_r_value(sema, re, span);
}

pub fn function_to_pointer(sema: &mut Sema, re: &mut ResolvedExpression) {
    let ResolvedType::Function { .. } = sema.types.get(re.ty.id) else { return };
    let to = QualifiedType::new(sema.types.pointer(re.ty), false, false);
    re.casts.push(ImplicitCast::new(CastKind::FunctionToPointer, to));
}

pub fn array_to_pointer(sema: &mut Sema, re: &mut ResolvedExpression) {
    let ResolvedType::Array { elem, .. } = sema.types.get(re.ty.id) else { return };
    let to = QualifiedType::new(sema.types.pointer(*elem), false, false);
    re.casts.push(ImplicitCast::new(CastKind::ArrayToPointer, to))
}

pub fn l_to_r_value(sema: &mut Sema, re: &mut ResolvedExpression, span: &Span) {
    if !matches!(re.kind, ExpressionKind::LValue) {
        return;
    }
    let ty = sema.types.get(re.ty.id);
    if matches!(ty, ResolvedType::Array { .. } | ResolvedType::Function { .. }) {
        return;
    }
    if !ty.is_complete(&sema.tags) {
        sema.add_diag(Diag::only_diag(Diagnosis::IncompleteType), span);
    }
    let to = QualifiedType::new(re.ty.id, false, false);
    re.casts.push(ImplicitCast::new(CastKind::LValueToRValue, to));
}

// 6.2.1.1
// A char, a short int. or an int bit-field. or their signed or unsigned varieties. or an
// enumeration type. may be used in an expression wherever an int or unsigned int may be
// used If an int can represent all values of the original type. the value is converted to an inf;
// otherwise, it is converted to an unsigned int. These are called the integral p~wnotions.”
// All other arithmetic types are unchanged by the integral promotions.
pub fn promote<'a>(sema: &Sema, re: &'a mut ResolvedExpression) -> &'a mut ResolvedExpression {
    use ResolvedType::*;

    let qty = re.casted_ty();
    match sema.types.get(qty.id) {
        Char | SignedChar | UnsignedChar | Short | UnsignedShort => (),
        &Tag(id) if sema.tags.get(id).kind == ast::Tag::Enum => (),
        _ => return re,
    }

    let kind = CastKind::IntegerPromotion;
    let to = QualifiedType::new(sema.builtins.int, false, false);
    re.casts.push(ImplicitCast::new(kind, to));
    re
}

// 6.2.2.1 If the lvalue has qualified type, the value has the unqualified version of the type
// of the lvalue: otherwise, the value has the type of the lvalue.
fn convert_type(ty: ResolvedTypeId) -> QualifiedType {
    QualifiedType::new(ty, false, false)
}

fn convert(sema: &Sema, from_re: &mut ResolvedExpression, to_id: ResolvedTypeId) {
    let from = from_re.casted_ty();
    match (sema.types.get(from.id), sema.types.get(to_id)) {
        // from.ty == to.ty                      -> nothing
        _ if from.id == to_id => (),
        // both arithmetic                       -> num_conv           // 6.2.1.2-4
        (f, t) if f.is_arithmetic(sema) && t.is_arithmetic(sema) => num_conv(sema, from_re, to_id),
        // to is void                            -> ToVoid             // 6.3.4
        (_, ResolvedType::Void) => (),
        // re is a null pointer constant, to is a pointer -> NullPointer  // 6.2.2.3
        (ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => (),
        // both pointers                         -> PointerConversion  // 6.3.16.1
        (ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => (),
        _ => unreachable!(),
    }
}
fn num_conv(sema: &Sema, from_re: &mut ResolvedExpression, to_id: ResolvedTypeId) {
    let from = from_re.casted_ty().id;
    if from == to_id {
        return;
    }
    let from = sema.types.get(from);
    let to = sema.types.get(to_id);
    let kind = match (from.is_integral(sema), to.is_integral(sema)) {
        (true, true) => CastKind::IntegerConversion,    // 6.2.1.2
        (true, false) => CastKind::IntegerToFloating,   // 6.2.1.3
        (false, true) => CastKind::FloatingToInteger,   // 6.2.1.4
        (false, false) => CastKind::FloatingConversion, // 6.2.1.4
    };
    from_re.casts.push(ImplicitCast::new(kind, convert_type(to_id)))
}

// 6.2.1.5 Usual arithmetic conversions
pub fn usual_arithmetic<'a>(
    sema: &mut Sema,
    lhs: &'a mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
) -> &'a mut ResolvedExpression {
    use ResolvedType::*;

    let l = sema.types.get(lhs.casted_ty().id);
    let r = sema.types.get(rhs.casted_ty().id);
    // 6.3.5 If both operands have arithmetic type, the usual arithmetic conversions are performed
    if !l.is_arithmetic(sema) || !r.is_arithmetic(sema) {
        return lhs;
    }
    // 6.2.1.5 Otherwise, the integral promotions are performed on both operands.
    if l.is_integral(sema) && r.is_integral(sema) {
        promote(sema, lhs);
        promote(sema, rhs);
    }
    if lhs.casted_ty().id == rhs.casted_ty().id {
        return lhs;
    }
    let to = match (l, r) {
        (LongDouble, _) | (_, LongDouble) => sema.builtins.long_double,
        (Double, _) | (_, Double) => sema.builtins.double,
        (Float, _) | (_, Float) => sema.builtins.float,
        (_, UnsignedLong) | (UnsignedLong, _) => sema.builtins.unsigned_long,
        (UnsignedInt, Long) | (Long, UnsignedInt) if sema.target.long.size > sema.target.int.size => sema.builtins.long,
        (UnsignedInt, Long) | (Long, UnsignedInt) => sema.builtins.unsigned_long,
        (_, Long) | (Long, _) => sema.builtins.long,
        (_, UnsignedInt) | (UnsignedInt, _) => sema.builtins.unsigned_int,
        _ => sema.builtins.int,
    };
    num_conv(sema, lhs, to);
    num_conv(sema, rhs, to);
    lhs
}

// fn assignment_conversion(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) {
//     let l = sema.types.get(lhs.value_ty().ty);
//     let r = sema.types.get(rhs.value_ty().ty);
//
//     let _ = l.is_arithmetic(&sema.tags) && r.is_arithmetic(&sema.tags);
// }

pub fn default_argument_promotions(sema: &Sema, re: &mut ResolvedExpression) {
    if re.casted_ty().id == sema.builtins.float {
        num_conv(sema, re, sema.builtins.double);
    } else {
        promote(sema, re);
    }
}

// NullPointer,       // 6.2.2.3
// ToVoid,            // 6.3.4
// PointerConversion, // 6.3.16.1

// convert is the dispatcher — the only place a (from, to) pair turns into a CastKind:
// pub fn convert(sema: &mut Sema, re: &mut ResolvedExpression, to: QualifiedType) {}

// fn assignment_conversion() {}
