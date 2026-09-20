use cc1::ast::ConstValue;
use cc1::semantic::{QualifiedType, ResolvedType, ResolvedTypeId, TagDefId};

use crate::common::{Unit, repr};

fn layout(ty: &ResolvedType) -> Option<(u32, u32)> {
    ty.layout().map(|l| (l.size, l.align))
}

fn pointer_to_int() -> ResolvedType {
    ResolvedType::Pointer(QualifiedType::new(ResolvedTypeId::from(0usize), false, false))
}

fn array_of_int() -> ResolvedType {
    ResolvedType::Array { elem: QualifiedType::new(ResolvedTypeId::from(0usize), false, false), len: Some(4) }
}

fn tag() -> ResolvedType {
    ResolvedType::Tag(TagDefId::from(0usize))
}

#[test]
fn scalar_layouts() {
    assert_eq!(layout(&ResolvedType::Char), Some((1, 1)));
    assert_eq!(layout(&ResolvedType::SignedChar), Some((1, 1)));
    assert_eq!(layout(&ResolvedType::UnsignedChar), Some((1, 1)));
    assert_eq!(layout(&ResolvedType::Short), Some((2, 2)));
    assert_eq!(layout(&ResolvedType::UnsignedShort), Some((2, 2)));
    assert_eq!(layout(&ResolvedType::Int), Some((4, 4)));
    assert_eq!(layout(&ResolvedType::UnsignedInt), Some((4, 4)));
    assert_eq!(layout(&ResolvedType::Long), Some((4, 4)));
    assert_eq!(layout(&ResolvedType::UnsignedLong), Some((4, 4)));
    assert_eq!(layout(&ResolvedType::Float), Some((4, 4)));
    assert_eq!(layout(&ResolvedType::Double), Some((8, 4)));
    assert_eq!(layout(&ResolvedType::LongDouble), Some((12, 4)));
}

#[test]
fn pointers_use_the_pointer_layout() {
    assert_eq!(layout(&pointer_to_int()), Some((4, 4)));
}

#[test]
fn void_arrays_and_tags_have_no_scalar_layout() {
    assert!(layout(&ResolvedType::Void).is_none());
    assert!(layout(&array_of_int()).is_none());
    assert!(layout(&tag()).is_none());
    assert!(ResolvedType::Void.bits().is_none());
}

#[test]
fn bits_follow_the_layout() {
    assert_eq!(ResolvedType::Char.bits(), Some(8));
    assert_eq!(ResolvedType::Short.bits(), Some(16));
    assert_eq!(ResolvedType::Int.bits(), Some(32));
    assert_eq!(ResolvedType::Long.bits(), Some(32));
}

#[test]
fn integral_types_are_the_integer_types() {
    assert!(ResolvedType::Char.is_integer());
    assert!(ResolvedType::UnsignedLong.is_integer());
    assert!(!ResolvedType::Float.is_integer());
    assert!(!ResolvedType::Double.is_integer());
    assert!(!ResolvedType::LongDouble.is_integer());
    assert!(!ResolvedType::Void.is_integer());
    assert!(!pointer_to_int().is_integer());
    assert!(!tag().is_integer());
}

#[test]
fn plain_char_is_signed() {
    assert!(ResolvedType::Char.is_signed());
    assert!(ResolvedType::SignedChar.is_signed());
    assert!(!ResolvedType::UnsignedChar.is_signed());
    assert!(!ResolvedType::UnsignedShort.is_signed());
    assert!(!ResolvedType::UnsignedInt.is_signed());
    assert!(!ResolvedType::UnsignedLong.is_signed());
    assert!(ResolvedType::Short.is_signed());
    assert!(ResolvedType::Int.is_signed());
    assert!(ResolvedType::Long.is_signed());
}

#[test]
fn value_ranges() {
    assert_eq!(ResolvedType::Char.max_value(), Some(127));
    assert_eq!(ResolvedType::Char.min_value(), Some(-128));
    assert_eq!(ResolvedType::UnsignedChar.max_value(), Some(255));
    assert_eq!(ResolvedType::UnsignedChar.min_value(), Some(0));
    assert_eq!(ResolvedType::Short.max_value(), Some(32767));
    assert_eq!(ResolvedType::Short.min_value(), Some(-32768));
    assert_eq!(ResolvedType::Int.max_value(), Some(2147483647));
    assert_eq!(ResolvedType::Int.min_value(), Some(-2147483648));
    assert_eq!(ResolvedType::UnsignedInt.max_value(), Some(4294967295));
    assert_eq!(ResolvedType::Long.max_value(), Some(2147483647));
    assert_eq!(ResolvedType::UnsignedLong.max_value(), Some(4294967295));
}

#[test]
fn ranges_are_defined_for_integer_types_only() {
    assert_eq!(ResolvedType::Float.max_value(), None);
    assert_eq!(ResolvedType::Double.min_value(), None);
    assert_eq!(ResolvedType::Void.max_value(), None);
    assert_eq!(pointer_to_int().min_value(), None);
}

#[test]
fn cast_narrows_and_retypes() {
    assert_eq!(repr(ResolvedType::Int.cast(ConstValue::Long(4294967297))), "Int(1)");
    assert_eq!(repr(ResolvedType::Char.cast(ConstValue::Int(321))), "Int(65)");
    assert_eq!(repr(ResolvedType::UnsignedChar.cast(ConstValue::Int(-1))), "Int(255)");
    assert_eq!(repr(ResolvedType::UnsignedInt.cast(ConstValue::Int(-1))), "UnsignedInt(4294967295)");
    assert_eq!(repr(ResolvedType::Long.cast(ConstValue::UnsignedLong(4294967295))), "Long(-1)");
}

#[test]
fn cast_is_defined_for_integer_types_only() {
    assert_eq!(repr(ResolvedType::Float.cast(ConstValue::Int(1))), "None");
    assert_eq!(repr(ResolvedType::Void.cast(ConstValue::Int(1))), "None");
    assert_eq!(repr(pointer_to_int().cast(ConstValue::Int(1))), "None");
}

fn probe(ty: &str) -> String {
    let unit = Unit::compile(&format!("enum probe {{ PROBE = sizeof({ty}) }};"));
    assert!(unit.accepts(), "sizeof({ty}):\n{}", unit.render());
    unit.enumerators().into_iter().find(|(name, _)| name == "PROBE").expect("the probe enumerator").1
}

#[test]
fn compiled_sizes() {
    assert_eq!(probe("long"), "4");
    assert_eq!(probe("int *"), "4");
    assert_eq!(probe("long double"), "12");
}

#[test]
fn ptrdiff_t_is_int() {
    let subtract = "int *p, *q; enum probe { PROBE = sizeof(p - q) };";
    let unit = Unit::compile(subtract);
    assert!(unit.accepts(), "{}", unit.render());
    assert_eq!(unit.sema.builtins.ptrdiff_t, unit.sema.builtins.int);
}
