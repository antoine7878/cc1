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
    FloatingToInteger,  // 6.2.1.4
    FloatingConversion, // 6.2.1.4
    FunctionToPointer,  // 6.2.2.1
    LValueToRValue,     // 6.2.2.1
    ArrayToPointer,     // 6.2.2.1
    NullPointer,        // 6.2.2.3
    ToVoid,             // 6.3.4
    PointerConversion,  // 6.3.16.1
}

// 6.2.2.1 an lvalue that does not have array type is converted to the value stored in the
// designated object (and is no longer an lvalue). If the lvalue has qualified type, the value
// has the unqualified version of the type of the lvalue.
// 6.2.2.1 If the lvalue has an incomplete type and does not have array type, the behavior is undefined.
pub fn l_to_r_value(sema: &mut Sema, re: &mut ResolvedExpression, span: &Span) {
    if !matches!(re.kind, ExpressionKind::LValue) {
        return;
    }
    let ty = sema.types.get(re.ty.ty);
    if matches!(ty, ResolvedType::Array { .. } | ResolvedType::Function { .. }) {
        return;
    }
    if !ty.is_complete(&sema.tags) {
        sema.add_diag(Diag::only_diag(Diagnosis::IncompleteType), span);
    }
    let to = QualifiedType::new(re.ty.ty, false, false);
    re.casts.push(ImplicitCast::new(CastKind::LValueToRValue, to));
}

// 6.2.1.1
// A char, a short int. or an int bit-field. or their signed or unsigned varieties. or an
// enumeration type. may be used in an expression wherever an int or unsigned int may be
// used If an int can represent all values of the original type. the value is converted to an inf;
// otherwise, it is converted to an unsigned int. These are called the integral p~wnotions.”
// All other arithmetic types are unchanged by the integral promotions.
pub fn int_promote(sema: &Sema, re: &mut ResolvedExpression) {
    use ResolvedType::*;

    let qty = re.value_ty();
    match sema.types.get(qty.ty) {
        Char | SignedChar | UnsignedChar | Short | UnsignedShort => (),
        &Tag(id) if sema.tags.get(id).kind == ast::Tag::Enum => (),
        _ => return,
    }

    let kind = CastKind::IntegerPromotion;
    let to = QualifiedType::new(sema.builtins.int, false, false);
    re.casts.push(ImplicitCast::new(kind, to));
}

fn convert_type(re: &ResolvedExpression, ty: ResolvedTypeId) -> QualifiedType {
    let mut qty = re.ty;
    qty.ty = ty;
    qty
}

fn num_conv(sema: &Sema, re: &mut ResolvedExpression, ty_id: ResolvedTypeId) {
    let from = sema.types.get(re.ty.ty);
    let to = sema.types.get(ty_id);
    let kind = match (from.is_integer(), to.is_integer()) {
        (true, true) => CastKind::IntegerConversion,    // 6.2.1.2
        (true, false) => CastKind::IntegerToFloating,   // 6.2.1.3
        (false, true) => CastKind::FloatingToInteger,   // 6.2.1.4
        (false, false) => CastKind::FloatingConversion, // 6.2.1.4
    };
    let to = convert_type(re, ty_id);
    re.casts.push(ImplicitCast::new(kind, to))
}

// 6.2.1.5 Usual arithmetic conversions
pub fn usual_arithmetic(sema: &mut Sema, lhs: &mut ResolvedExpression, rhs: &mut ResolvedExpression) {
    use ResolvedType::*;

    let l = sema.types.get(lhs.ty.ty);
    let r = sema.types.get(rhs.ty.ty);

    if l.is_integer() && r.is_integer() {
        int_promote(sema, lhs);
        int_promote(sema, rhs);
    }
    let to = match (l, r) {
        (l, r) if l == r => return,
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

// pub fn promote(sema: &mut Sema, ctx: &Context, expr: &mut Typed) {
// }

// convert is the dispatcher — the only place a (from, to) pair turns into a CastKind:
// pub fn convert(sema: &mut Sema, expr: &mut Typed, to: QualifiedType) {}

// fn promote() {}
// fn lvalue_conversion() {}
// fn usual_arithmetic() {}
// fn assignment_conversion() {}
// fn default_argument_promotions() {}
