use crate::common::{Unit, repr};
use cc1::ast::{F80, Value};
use cc1::semantic::{QualifiedType, ResolvedType, ResolvedTypeId, TagDefId};
use cc1::target::{ARM64_DARWIN, FloatFormat, I386, Target, X86_64};

fn layout(target: &Target, ty: &ResolvedType) -> Option<(u32, u32)> {
    target.layout(ty).map(|l| (l.size, l.align))
}

fn pointer_to_int() -> ResolvedType {
    ResolvedType::Pointer(QualifiedType::new(ResolvedTypeId::from(0usize), false, false))
}

fn array_of_int() -> ResolvedType {
    ResolvedType::Array {
        elem: QualifiedType::new(ResolvedTypeId::from(0usize), false, false),
        len: Some(4),
    }
}

fn tag() -> ResolvedType {
    ResolvedType::Tag(TagDefId::from(0usize))
}

#[test]
fn default_target_is_x86_64() {
    assert_eq!(Target::default().name, "x86_64");
    assert_eq!(I386.name, "i386");
    assert_eq!(X86_64.name, "x86_64");
}

#[test]
fn i386_scalar_layouts() {
    assert_eq!(layout(&I386, &ResolvedType::Char), Some((1, 1)));
    assert_eq!(layout(&I386, &ResolvedType::SignedChar), Some((1, 1)));
    assert_eq!(layout(&I386, &ResolvedType::UnsignedChar), Some((1, 1)));
    assert_eq!(layout(&I386, &ResolvedType::Short), Some((2, 2)));
    assert_eq!(layout(&I386, &ResolvedType::UnsignedShort), Some((2, 2)));
    assert_eq!(layout(&I386, &ResolvedType::Int), Some((4, 4)));
    assert_eq!(layout(&I386, &ResolvedType::UnsignedInt), Some((4, 4)));
    assert_eq!(layout(&I386, &ResolvedType::Long), Some((4, 4)));
    assert_eq!(layout(&I386, &ResolvedType::UnsignedLong), Some((4, 4)));
    assert_eq!(layout(&I386, &ResolvedType::Float), Some((4, 4)));
    assert_eq!(layout(&I386, &ResolvedType::Double), Some((8, 4)));
    assert_eq!(layout(&I386, &ResolvedType::LongDouble), Some((12, 4)));
}

#[test]
fn x86_64_scalar_layouts() {
    assert_eq!(layout(&X86_64, &ResolvedType::Char), Some((1, 1)));
    assert_eq!(layout(&X86_64, &ResolvedType::Short), Some((2, 2)));
    assert_eq!(layout(&X86_64, &ResolvedType::Int), Some((4, 4)));
    assert_eq!(layout(&X86_64, &ResolvedType::Long), Some((8, 8)));
    assert_eq!(layout(&X86_64, &ResolvedType::UnsignedLong), Some((8, 8)));
    assert_eq!(layout(&X86_64, &ResolvedType::Float), Some((4, 4)));
    assert_eq!(layout(&X86_64, &ResolvedType::Double), Some((8, 8)));
    assert_eq!(layout(&X86_64, &ResolvedType::LongDouble), Some((16, 16)));
}

#[test]
fn pointers_use_the_pointer_layout() {
    assert_eq!(layout(&I386, &pointer_to_int()), Some((4, 4)));
    assert_eq!(layout(&X86_64, &pointer_to_int()), Some((8, 8)));
}

#[test]
fn void_arrays_and_tags_have_no_scalar_layout() {
    assert!(layout(&I386, &ResolvedType::Void).is_none());
    assert!(layout(&I386, &array_of_int()).is_none());
    assert!(layout(&X86_64, &array_of_int()).is_none());
    assert!(layout(&I386, &tag()).is_none());
    assert!(I386.bits(&ResolvedType::Void).is_none());
}

#[test]
fn bits_follow_the_layout() {
    assert_eq!(I386.bits(&ResolvedType::Char), Some(8));
    assert_eq!(I386.bits(&ResolvedType::Short), Some(16));
    assert_eq!(I386.bits(&ResolvedType::Int), Some(32));
    assert_eq!(I386.bits(&ResolvedType::Long), Some(32));
    assert_eq!(X86_64.bits(&ResolvedType::Long), Some(64));
}

#[test]
fn integral_types_are_the_integer_types() {
    assert!(I386.is_integral(&ResolvedType::Char));
    assert!(I386.is_integral(&ResolvedType::UnsignedLong));
    assert!(!I386.is_integral(&ResolvedType::Float));
    assert!(!I386.is_integral(&ResolvedType::Double));
    assert!(!I386.is_integral(&ResolvedType::LongDouble));
    assert!(!I386.is_integral(&ResolvedType::Void));
    assert!(!I386.is_integral(&pointer_to_int()));
    assert!(!I386.is_integral(&tag()));
}

#[test]
fn plain_char_signedness_follows_the_target() {
    const { assert!(I386.char_signed) }
    assert!(I386.is_signed(&ResolvedType::Char));
    assert!(I386.is_signed(&ResolvedType::SignedChar));
    assert!(!I386.is_signed(&ResolvedType::UnsignedChar));
    assert!(!I386.is_signed(&ResolvedType::UnsignedShort));
    assert!(!I386.is_signed(&ResolvedType::UnsignedInt));
    assert!(!I386.is_signed(&ResolvedType::UnsignedLong));
    assert!(I386.is_signed(&ResolvedType::Short));
    assert!(I386.is_signed(&ResolvedType::Int));
    assert!(I386.is_signed(&ResolvedType::Long));
}

#[test]
fn i386_value_ranges() {
    assert_eq!(I386.max_value(&ResolvedType::Char), Some(127));
    assert_eq!(I386.min_value(&ResolvedType::Char), Some(-128));
    assert_eq!(I386.max_value(&ResolvedType::UnsignedChar), Some(255));
    assert_eq!(I386.min_value(&ResolvedType::UnsignedChar), Some(0));
    assert_eq!(I386.max_value(&ResolvedType::Short), Some(32767));
    assert_eq!(I386.min_value(&ResolvedType::Short), Some(-32768));
    assert_eq!(I386.max_value(&ResolvedType::Int), Some(2147483647));
    assert_eq!(I386.min_value(&ResolvedType::Int), Some(-2147483648));
    assert_eq!(I386.max_value(&ResolvedType::UnsignedInt), Some(4294967295));
    assert_eq!(I386.max_value(&ResolvedType::Long), Some(2147483647));
    assert_eq!(I386.max_value(&ResolvedType::UnsignedLong), Some(4294967295));
}

#[test]
fn x86_64_value_ranges() {
    assert_eq!(X86_64.max_value(&ResolvedType::Long), Some(i64::MAX as u64));
    assert_eq!(X86_64.min_value(&ResolvedType::Long), Some(i64::MIN));
    assert_eq!(X86_64.max_value(&ResolvedType::UnsignedLong), Some(u64::MAX));
    assert_eq!(X86_64.min_value(&ResolvedType::UnsignedLong), Some(0));
}

#[test]
fn ranges_are_defined_for_integer_types_only() {
    assert_eq!(I386.max_value(&ResolvedType::Float), None);
    assert_eq!(I386.min_value(&ResolvedType::Double), None);
    assert_eq!(I386.max_value(&ResolvedType::Void), None);
    assert_eq!(I386.min_value(&pointer_to_int()), None);
}

#[test]
fn truncate_wraps_to_the_target_width() {
    assert_eq!(I386.truncate(300, &ResolvedType::Char), Some(44));
    assert_eq!(I386.truncate(255, &ResolvedType::Char), Some(-1));
    assert_eq!(I386.truncate(-1, &ResolvedType::UnsignedChar), Some(255));
    assert_eq!(I386.truncate(65536, &ResolvedType::Short), Some(0));
    assert_eq!(I386.truncate(4294967296, &ResolvedType::Int), Some(0));
    assert_eq!(I386.truncate(-1, &ResolvedType::UnsignedInt), Some(4294967295));
    assert_eq!(I386.truncate(1, &ResolvedType::Int), Some(1));
    assert_eq!(I386.truncate(-1, &ResolvedType::Int), Some(-1));
}

#[test]
fn truncate_at_full_width_is_the_identity() {
    assert_eq!(X86_64.truncate(-1, &ResolvedType::Long), Some(-1));
    assert_eq!(X86_64.truncate(i64::MIN, &ResolvedType::Long), Some(i64::MIN));
    assert_eq!(X86_64.truncate(-1, &ResolvedType::UnsignedLong), Some(-1));
}

#[test]
fn truncate_is_defined_for_integer_types_only() {
    assert_eq!(I386.truncate(1, &ResolvedType::Float), None);
    assert_eq!(I386.truncate(1, &ResolvedType::Void), None);
}

#[test]
fn cast_narrows_and_retypes() {
    assert_eq!(repr(I386.cast(&ResolvedType::Int, Value::Long(4294967297))), "Int(1)");
    assert_eq!(repr(I386.cast(&ResolvedType::Char, Value::Int(321))), "Int(65)");
    assert_eq!(repr(I386.cast(&ResolvedType::UnsignedChar, Value::Int(-1))), "Int(255)");
    assert_eq!(
        repr(I386.cast(&ResolvedType::UnsignedInt, Value::Int(-1))),
        "UnsignedInt(4294967295)"
    );
    assert_eq!(
        repr(I386.cast(&ResolvedType::Long, Value::UnsignedLong(4294967295))),
        "Long(-1)"
    );
    assert_eq!(
        repr(X86_64.cast(&ResolvedType::Long, Value::UnsignedLong(4294967295))),
        "Long(4294967295)"
    );
    assert_eq!(
        repr(X86_64.cast(&ResolvedType::UnsignedLong, Value::Int(-1))),
        "UnsignedLong(18446744073709551615)"
    );
}

#[test]
fn cast_is_defined_for_integer_types_only() {
    assert_eq!(repr(I386.cast(&ResolvedType::Float, Value::Int(1))), "None");
    assert_eq!(repr(I386.cast(&ResolvedType::Void, Value::Int(1))), "None");
    assert_eq!(repr(I386.cast(&pointer_to_int(), Value::Int(1))), "None");
}

#[test]
fn value_size_follows_the_value_type() {
    assert_eq!(I386.value_size(Value::Int(0)), 4);
    assert_eq!(I386.value_size(Value::UnsignedInt(0)), 4);
    assert_eq!(I386.value_size(Value::Long(0)), 4);
    assert_eq!(I386.value_size(Value::Float(0.0)), 4);
    assert_eq!(I386.value_size(Value::Double(0.0)), 8);
    assert_eq!(I386.value_size(Value::LongDouble(F80::from(0.0))), 12);
    assert_eq!(X86_64.value_size(Value::Long(0)), 8);
    assert_eq!(X86_64.value_size(Value::LongDouble(F80::from(0.0))), 16);
}

#[test]
fn abi_types_follow_the_target() {
    assert_eq!(I386.size_t, ResolvedType::UnsignedInt);
    assert_eq!(I386.ptrdiff_t, ResolvedType::Int);
    assert_eq!(I386.wchar_t, ResolvedType::Long);
    assert_eq!(X86_64.size_t, ResolvedType::UnsignedLong);
    assert_eq!(X86_64.ptrdiff_t, ResolvedType::Long);
    assert_eq!(X86_64.wchar_t, ResolvedType::Int);
}

fn probe(target: Target, ty: &str) -> String {
    let name = target.name;
    let unit = Unit::compile_for(target, &format!("enum probe {{ PROBE = sizeof({ty}) }};"));
    assert!(unit.accepts(), "sizeof({ty}) on {name}:\n{}", unit.render());
    unit.variants()
        .into_iter()
        .find(|(name, _)| name == "PROBE")
        .expect("the probe variant")
        .1
}

#[test]
fn the_selected_target_drives_compiled_sizes() {
    assert_eq!(probe(I386, "long"), "4");
    assert_eq!(probe(X86_64, "long"), "8");
    assert_eq!(probe(I386, "int *"), "4");
    assert_eq!(probe(X86_64, "int *"), "8");
    assert_eq!(probe(I386, "long double"), "12");
    assert_eq!(probe(X86_64, "long double"), "16");
}

#[test]
fn the_selected_target_drives_ptrdiff_t() {
    let subtract = "int *p, *q; enum probe { PROBE = sizeof(1 ? 0 : 0) };";
    assert!(Unit::compile_for(I386, subtract).accepts());
    assert!(Unit::compile_for(X86_64, subtract).accepts());
    assert_eq!(I386.ptrdiff_t, ResolvedType::Int);
    assert_eq!(X86_64.ptrdiff_t, ResolvedType::Long);
}

#[test]
fn arm64_scalar_layouts() {
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::Char), Some((1, 1)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::Short), Some((2, 2)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::Int), Some((4, 4)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::Long), Some((8, 8)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::UnsignedLong), Some((8, 8)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::Float), Some((4, 4)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::Double), Some((8, 8)));
    assert_eq!(layout(&ARM64_DARWIN, &ResolvedType::LongDouble), Some((8, 8)));
    assert_eq!(layout(&ARM64_DARWIN, &pointer_to_int()), Some((8, 8)));
}

#[test]
fn arm64_abi_types_and_ranges() {
    assert_eq!(ARM64_DARWIN.name, "arm64");
    assert_eq!(ARM64_DARWIN.size_t, ResolvedType::UnsignedLong);
    assert_eq!(ARM64_DARWIN.ptrdiff_t, ResolvedType::Long);
    assert_eq!(ARM64_DARWIN.wchar_t, ResolvedType::Int);
    assert!(ARM64_DARWIN.char_signed);
    assert!(ARM64_DARWIN.is_signed(&ResolvedType::Char));
    assert_eq!(ARM64_DARWIN.max_value(&ResolvedType::Long), Some(i64::MAX as u64));
    assert_eq!(ARM64_DARWIN.min_value(&ResolvedType::Long), Some(i64::MIN));
    assert_eq!(ARM64_DARWIN.value_size(Value::LongDouble(F80::from(0.0))), 8);
}

#[test]
fn long_double_format_follows_the_target() {
    assert_eq!(I386.long_double_format, FloatFormat::X87);
    assert_eq!(X86_64.long_double_format, FloatFormat::X87);
    assert_eq!(ARM64_DARWIN.long_double_format, FloatFormat::Ieee64);
    assert_eq!(FloatFormat::X87.llvm(), "x86_fp80");
    assert_eq!(FloatFormat::Ieee64.llvm(), "double");
}

#[test]
fn each_target_carries_its_module_header() {
    assert_eq!(I386.triple, "i386-pc-linux-gnu");
    assert_eq!(X86_64.triple, "x86_64-pc-linux-gnu");
    assert_eq!(ARM64_DARWIN.triple, "arm64-apple-macosx26.0.0");
    assert!(I386.datalayout.starts_with("e-m:e-p:32:32-"));
    assert!(X86_64.datalayout.contains("f80:128"));
    assert_eq!(
        ARM64_DARWIN.datalayout,
        "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32"
    );
}

#[test]
fn arm64_compiled_sizes() {
    assert_eq!(probe(ARM64_DARWIN, "long"), "8");
    assert_eq!(probe(ARM64_DARWIN, "int *"), "8");
    assert_eq!(probe(ARM64_DARWIN, "double"), "8");
    assert_eq!(probe(ARM64_DARWIN, "long double"), "8");
}

#[test]
fn the_float_format_rounds_to_its_own_precision() {
    let tenth = F80::from("0.1");
    assert_eq!(FloatFormat::X87.round(tenth), tenth);
    assert_eq!(FloatFormat::Ieee64.round(tenth), F80::from(0.1));
    assert_ne!(FloatFormat::Ieee64.round(tenth), tenth);
    assert_eq!(FloatFormat::Ieee64.round(F80::from(1.5)), F80::from(1.5));
}
