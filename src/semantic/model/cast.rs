use crate::arena::ResolveWith;
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
    PointerToInteger,  // 6.3.4
    IntegerToPointer,  // 6.3.4
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
    let ResolvedType::Function { .. } = re.ty.id.resolve(sema) else { return };
    let to = QualifiedType::new(sema.types.pointer(re.ty), false, false);
    re.casts.push(ImplicitCast::new(CastKind::FunctionToPointer, to));
}

pub fn array_to_pointer(sema: &mut Sema, re: &mut ResolvedExpression) {
    let ResolvedType::Array { elem, .. } = re.ty.id.resolve(sema) else { return };
    let to = QualifiedType::new(sema.types.pointer(*elem), false, false);
    re.casts.push(ImplicitCast::new(CastKind::ArrayToPointer, to))
}

pub fn l_to_r_value(sema: &mut Sema, re: &mut ResolvedExpression, span: &Span) {
    if !matches!(re.kind, ExpressionKind::LValue) {
        return;
    }
    let ty = re.ty.id.resolve(sema);
    if matches!(ty, ResolvedType::Array { .. } | ResolvedType::Function { .. }) {
        return;
    }
    if !ty.is_complete(sema) {
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
pub fn promote(sema: &Sema, re: &mut ResolvedExpression) {
    use ResolvedType::*;

    let qty = re.casted_ty();
    match qty.id.resolve(sema) {
        Char | SignedChar | UnsignedChar | Short | UnsignedShort => (),
        &Tag(id) if id.resolve(sema).kind == ast::Tag::Enum => (),
        _ => return,
    }

    let kind = CastKind::IntegerPromotion;
    let to = QualifiedType::new(sema.builtins.int, false, false);
    re.casts.push(ImplicitCast::new(kind, to));
}

// 6.2.2.1 If the lvalue has qualified type, the value has the unqualified version of the type
// of the lvalue: otherwise, the value has the type of the lvalue.
fn convert_type(ty: ResolvedTypeId) -> QualifiedType {
    QualifiedType::new(ty, false, false)
}

pub fn convert(sema: &Sema, from_re: &mut ResolvedExpression, to_id: ResolvedTypeId, is_null_ptr: bool) {
    let from = from_re.casted_ty();
    let implicit_cast = match (from.id.resolve(sema), to_id.resolve(sema)) {
        _ if from.id == to_id => return,
        (f, t) if f.is_arithmetic(sema) && t.is_arithmetic(sema) => return num_conv(sema, from_re, to_id),
        (_, ResolvedType::Void) => ImplicitCast::new(CastKind::ToVoid, convert_type(to_id)),
        (_, ResolvedType::Pointer(_)) if is_null_ptr => ImplicitCast::new(CastKind::NullPointer, convert_type(to_id)),
        (ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => {
            ImplicitCast::new(CastKind::PointerConversion, convert_type(to_id))
        }
        (ResolvedType::Pointer(_), t) if t.is_integral(sema) => {
            ImplicitCast::new(CastKind::PointerToInteger, convert_type(to_id))
        }
        (f, ResolvedType::Pointer(_)) if f.is_integral(sema) => {
            ImplicitCast::new(CastKind::IntegerToPointer, convert_type(to_id))
        }
        _ => unreachable!(),
    };
    from_re.casts.push(implicit_cast)
}
fn num_conv(sema: &Sema, from_re: &mut ResolvedExpression, to_id: ResolvedTypeId) {
    let from = from_re.casted_ty().id;
    if from == to_id {
        return;
    }
    let from = from.resolve(sema);
    let to = to_id.resolve(sema);
    let kind = match (from.is_integral(sema), to.is_integral(sema)) {
        (true, true) => CastKind::IntegerConversion,    // 6.2.1.2
        (true, false) => CastKind::IntegerToFloating,   // 6.2.1.3
        (false, true) => CastKind::FloatingToInteger,   // 6.2.1.4
        (false, false) => CastKind::FloatingConversion, // 6.2.1.4
    };
    from_re.casts.push(ImplicitCast::new(kind, convert_type(to_id)))
}

// 6.2.1.5 Usual arithmetic conversions
pub fn usual_arithmetic(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) {
    use ResolvedType::*;

    let l = lhs.casted_ty().id.resolve(sema);
    let r = rhs.casted_ty().id.resolve(sema);
    // 6.3.5 If both operands have arithmetic type, the usual arithmetic conversions are performed
    if !l.is_arithmetic(sema) || !r.is_arithmetic(sema) {
        return;
    }
    // 6.2.1.5 Otherwise, the integral promotions are performed on both operands.
    if l.is_integral(sema) && r.is_integral(sema) {
        promote(sema, lhs);
        promote(sema, rhs);
    }
    if lhs.casted_ty().id == rhs.casted_ty().id {
        return;
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
}

fn is_object_or_incomplete(sema: &Sema, ty: ResolvedTypeId) -> bool {
    !matches!(ty.resolve(sema), ResolvedType::Function { .. } | ResolvedType::Void)
}

fn assign_pointer(sema: &Sema, lp: &QualifiedType, rp: &QualifiedType) -> bool {
    lp.is_compatible_ignoring_qualifiers(sema, rp)
        || (lp.id == sema.builtins.void && is_object_or_incomplete(sema, rp.id))
        || (rp.id == sema.builtins.void && is_object_or_incomplete(sema, lp.id))
}

fn discarded_qualifier(lp: &QualifiedType, rp: &QualifiedType) -> ast::Qualifier {
    if rp.is_const && !lp.is_const { ast::Qualifier::Const } else { ast::Qualifier::Volatile }
}

pub fn assignment_conversion(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    is_null_ptr: bool,
) -> Result<QualifiedType, Diagnosis> {
    let l = lhs.ty.id.resolve(sema);
    let rhs_ty = rhs.casted_ty();
    let r = rhs_ty.id.resolve(sema);
    match (l, r) {
        (l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => (),
        (&ResolvedType::Tag(id), _) if !id.resolve(sema).is_enum() && lhs.ty.is_compatible(sema, &rhs_ty) => (),
        (ResolvedType::Pointer(lp), ResolvedType::Pointer(rp)) if assign_pointer(sema, lp, rp) => {
            if !lp.has_qualifiers_of(rp) {
                return Err(Diagnosis::DiscardedQualifiers(discarded_qualifier(lp, rp)));
            }
        }
        (ResolvedType::Pointer(_), _) if is_null_ptr => (),
        _ => return Err(Diagnosis::IncompatibleAssignementTypes),
    }
    convert(sema, rhs, lhs.ty.id, is_null_ptr);
    Ok(lhs.ty)
}

pub fn default_argument_promotions(sema: &Sema, re: &mut ResolvedExpression) {
    if re.casted_ty().id == sema.builtins.float {
        num_conv(sema, re, sema.builtins.double);
    } else {
        promote(sema, re);
    }
}

pub fn pointer_integer_arithmetic(
    sema: &mut Sema,
    pointer: &mut ResolvedExpression,
    intergral: &mut ResolvedExpression,
) -> Result<QualifiedType, Diagnosis> {
    let ResolvedType::Pointer(inner) = pointer.casted_ty().id.resolve(sema) else {
        return Err(Diagnosis::Poisoned);
    };
    match inner.id.resolve(sema) {
        t if !t.is_complete(sema) => Err(Diagnosis::InvalidOperand),
        ResolvedType::Function { .. } => Err(Diagnosis::InvalidOperand),
        _ => {
            promote(sema, intergral);
            Ok(pointer.casted_ty())
        }
    }
}

pub fn pointer_minus_pointer(
    sema: &Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
) -> Result<QualifiedType, Diagnosis> {
    let ResolvedType::Pointer(lp) = lhs.casted_ty().id.resolve(sema) else {
        return Err(Diagnosis::Poisoned);
    };
    let ResolvedType::Pointer(rp) = rhs.casted_ty().id.resolve(sema) else {
        return Err(Diagnosis::Poisoned);
    };
    let l = lp.id.resolve(sema);
    let r = rp.id.resolve(sema);
    if !l.is_object(sema) || !r.is_object(sema) {
        return Err(Diagnosis::InvalidOperand);
    }
    if !lp.is_compatible_ignoring_qualifiers(sema, rp) {
        return Err(Diagnosis::InvalidOperand);
    }
    Ok(QualifiedType::new(sema.builtins.ptrdiff_t, false, false))
}
