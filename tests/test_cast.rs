use cc1::ast::Tag;
use cc1::semantic::model::cast::{promote, usual_arithmetic};
use cc1::semantic::{CastKind, ExpressionKind, QualifiedType, ResolvedExpression, ResolvedType, ResolvedTypeId, Sema};

fn rvalue(ty: ResolvedTypeId) -> ResolvedExpression {
    ResolvedExpression::new(QualifiedType::new(ty, false, false), ExpressionKind::RValue)
}

fn kinds(re: &ResolvedExpression) -> Vec<CastKind> {
    re.casts.iter().map(|cast| cast.kind).collect()
}

fn enum_type(sema: &mut Sema) -> ResolvedTypeId {
    let id = sema.tags.declare(Tag::Enum, None);
    sema.tags.complete(id, Vec::new());
    sema.types.tag(id)
}

fn struct_type(sema: &mut Sema) -> ResolvedTypeId {
    let id = sema.tags.declare(Tag::Struct, None);
    sema.tags.complete(id, Vec::new());
    sema.types.tag(id)
}

fn convert(
    sema: &mut Sema,
    lhs: ResolvedTypeId,
    rhs: ResolvedTypeId,
) -> (ResolvedTypeId, Vec<CastKind>, Vec<CastKind>) {
    let (mut lhs, mut rhs) = (rvalue(lhs), rvalue(rhs));
    usual_arithmetic(sema, &mut lhs, &mut rhs);
    assert_eq!(
        lhs.casted_ty().id,
        rhs.casted_ty().id,
        "the usual arithmetic conversions must yield a common type"
    );
    (lhs.casted_ty().id, kinds(&lhs), kinds(&rhs))
}

// ---- 6.2.1.5 the floating ladder -----------------------------------------

#[test]
fn long_double_dominates() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.int, b.long_double);
    assert_eq!(to, b.long_double);
    assert_eq!(l, vec![CastKind::IntegerToFloating]);
    assert!(r.is_empty());
}

// gcc: sizeof(i + 2.0) == 8
#[test]
fn integer_and_double_meet_at_double() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.int, b.double);
    assert_eq!(to, b.double);
    assert_eq!(l, vec![CastKind::IntegerToFloating]);
    assert!(r.is_empty());
}

// gcc: sizeof(i + f) == 4
#[test]
fn integer_and_float_meet_at_float() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.float, b.int);
    assert_eq!(to, b.float);
    assert!(l.is_empty());
    assert_eq!(r, vec![CastKind::IntegerToFloating]);
}

// gcc: sizeof((float)1 + (double)1) == 8
#[test]
fn float_and_double_meet_at_double() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.float, b.double);
    assert_eq!(to, b.double);
    assert_eq!(l, vec![CastKind::FloatingConversion]);
    assert!(r.is_empty());
}

// ---- 6.2.1.5 the integral promotions are performed on both operands -------

// gcc: sizeof((char)1 + (char)1) == 4
#[test]
fn narrow_operands_of_the_same_type_are_still_promoted() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.char, b.char);
    assert_eq!(to, b.int);
    assert_eq!(l, vec![CastKind::IntegerPromotion]);
    assert_eq!(r, vec![CastKind::IntegerPromotion]);
}

// gcc: sizeof((char)1 + (short)1) == 4
#[test]
fn promotion_alone_reaches_the_common_type() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.char, b.short);
    assert_eq!(to, b.int);
    assert_eq!(l, vec![CastKind::IntegerPromotion]);
    assert_eq!(r, vec![CastKind::IntegerPromotion]);
}

// A floating operand suppresses the promotions: char converts straight to double.
#[test]
fn a_floating_operand_suppresses_the_promotions() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.char, b.double);
    assert_eq!(to, b.double);
    assert_eq!(l, vec![CastKind::IntegerToFloating]);
    assert!(r.is_empty());
}

// ---- 6.1.2.5 the enumerated types are integral types ----------------------

// gcc: sizeof((enum E)0 + 0) == 4, and `enum E { A = -1 }` promotes to a signed int.
#[test]
fn an_enumerated_operand_is_promoted_to_int() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let e = enum_type(&mut sema);
    let (to, l, r) = convert(&mut sema, e, b.int);
    assert_eq!(to, b.int);
    assert_eq!(l, vec![CastKind::IntegerPromotion]);
    assert!(r.is_empty());
}

#[test]
fn an_enumerated_operand_converts_as_an_integer_not_a_float() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let e = enum_type(&mut sema);
    let (to, l, r) = convert(&mut sema, e, b.long);
    assert_eq!(to, b.long);
    assert_eq!(l, vec![CastKind::IntegerPromotion, CastKind::IntegerConversion]);
    assert!(r.is_empty());
}

#[test]
fn int_promote_promotes_an_enumerated_type() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let e = enum_type(&mut sema);
    let mut re = rvalue(e);
    promote(&sema, &mut re);
    assert_eq!(kinds(&re), vec![CastKind::IntegerPromotion]);
    assert_eq!(re.casted_ty().id, b.int);
}

// ---- 6.2.1.5 the integer ladder ------------------------------------------

#[test]
fn unsigned_long_dominates_the_integer_ladder() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.int, b.unsigned_long);
    assert_eq!(to, b.unsigned_long);
    assert_eq!(l, vec![CastKind::IntegerConversion]);
    assert!(r.is_empty());
}

// i386: a long int cannot represent all the values of an unsigned int, so both operands are
// converted to unsigned long int.
// gcc: ((long)-1 + (unsigned int)0) > 0
#[test]
fn unsigned_int_and_long_meet_at_unsigned_long_on_i386() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    assert_eq!(sema.target.long.size, sema.target.int.size);
    let (to, l, r) = convert(&mut sema, b.unsigned_int, b.long);
    assert_eq!(to, b.unsigned_long);
    assert_eq!(l, vec![CastKind::IntegerConversion]);
    assert_eq!(r, vec![CastKind::IntegerConversion]);
}

#[test]
fn int_and_long_meet_at_long() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.int, b.long);
    assert_eq!(to, b.long);
    assert_eq!(l, vec![CastKind::IntegerConversion]);
    assert!(r.is_empty());
}

#[test]
fn int_and_unsigned_int_meet_at_unsigned_int() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.int, b.unsigned_int);
    assert_eq!(to, b.unsigned_int);
    assert_eq!(l, vec![CastKind::IntegerConversion]);
    assert!(r.is_empty());
}

// ---- an operand already at the common type is left alone ------------------

#[test]
fn operands_of_the_same_type_are_not_converted() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let (to, l, r) = convert(&mut sema, b.long, b.long);
    assert_eq!(to, b.long);
    assert!(l.is_empty());
    assert!(r.is_empty());
}

// ---- 6.3.5 the conversions apply only to arithmetic operands --------------

#[test]
fn a_non_arithmetic_operand_is_left_alone() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let s = struct_type(&mut sema);
    let (mut lhs, mut rhs) = (rvalue(s), rvalue(b.int));
    usual_arithmetic(&mut sema, &mut lhs, &mut rhs);
    assert!(lhs.casts.is_empty());
    assert!(rhs.casts.is_empty());
}

#[test]
fn a_pointer_operand_is_left_alone() {
    let mut sema = Sema::default();
    let b = sema.builtins;
    let p = sema.types.pointer(QualifiedType::new(b.int, false, false));
    let (mut lhs, mut rhs) = (rvalue(p), rvalue(b.int));
    usual_arithmetic(&mut sema, &mut lhs, &mut rhs);
    assert!(lhs.casts.is_empty());
    assert!(rhs.casts.is_empty());
}

// ---- 6.1.2.5 type classification -----------------------------------------

#[test]
fn an_enumerated_type_is_integral_and_arithmetic_but_not_an_integer() {
    let mut sema = Sema::default();
    let e = enum_type(&mut sema);
    let ty = sema.types.get(e);
    assert!(ty.is_integral(&sema));
    assert!(ty.is_arithmetic(&sema));
    assert!(!ty.is_integer());
    assert!(!ty.is_floating());
}

#[test]
fn a_struct_type_is_neither_integral_nor_arithmetic() {
    let mut sema = Sema::default();
    let s = struct_type(&mut sema);
    let ty = sema.types.get(s);
    assert!(!ty.is_integral(&sema));
    assert!(!ty.is_arithmetic(&sema));
}

#[test]
fn the_basic_types_keep_their_classification() {
    let sema = Sema::default();
    assert!(ResolvedType::Char.is_integral(&sema));
    assert!(ResolvedType::UnsignedLong.is_integral(&sema));
    assert!(!ResolvedType::Double.is_integral(&sema));
    assert!(ResolvedType::Double.is_arithmetic(&sema));
    assert!(!ResolvedType::Void.is_arithmetic(&sema));
}
