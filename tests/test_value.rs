use std::cmp::Ordering;

use cc1::ast::{BinaryOp, Fold, UnaryOp, Value};
use cc1::semantic::{Diag, Diagnosis, ResolvedType};
use cc1::target::{I386, X86_64};

trait FoldValue {
    fn fold_value(self) -> Value;
}
impl FoldValue for Value {
    fn fold_value(self) -> Value {
        self
    }
}
impl FoldValue for Diag<Value> {
    fn fold_value(self) -> Value {
        self.res
    }
}
impl FoldValue for Option<Value> {
    fn fold_value(self) -> Value {
        self.expect("conversion should succeed")
    }
}

fn repr(value: impl FoldValue) -> String {
    crate::common::repr(Some(value.fold_value()))
}

macro_rules! fold {
    ($name:ident, $method:ident($($arg:expr),* $(,)?), $expected:expr) => {
        #[test]
        fn $name() {
            let fold = Fold::new(&I386);
            assert_eq!(
                repr(fold.$method($($arg),*)),
                $expected,
                "{}",
                stringify!($method($($arg),*))
            );
        }
    };
    (ignore $reason:literal, $name:ident, $method:ident($($arg:expr),* $(,)?), $expected:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            let fold = Fold::new(&I386);
            assert_eq!(
                repr(fold.$method($($arg),*)),
                $expected,
                "{}",
                stringify!($method($($arg),*))
            );
        }
    };
}

macro_rules! fold_overflow {
    ($name:ident, $method:ident($($arg:expr),* $(,)?), $expected:expr) => {
        #[test]
        fn $name() {
            let Diag { res, diagnosis } = Fold::new(&I386).$method($($arg),*);
            assert_eq!(repr(res), $expected, "{}", stringify!($method($($arg),*)));
            assert!(
                matches!(diagnosis, Some(Diagnosis::ArithmeticOverflow)),
                "{} should report ArithmeticOverflow, got {diagnosis:?}",
                stringify!($method($($arg),*))
            );
        }
    };
}

constant!(literal_zero, "0", "Int(0)");
constant!(literal_decimal, "42", "Int(42)");
constant!(literal_int_max, "2147483647", "Int(2147483647)");
constant!(literal_decimal_above_int_max, "2147483648", "UnsignedLong(2147483648)");
constant!(literal_octal, "010", "Int(8)");
constant!(literal_octal_max, "017777777777", "Int(2147483647)");
constant!(literal_hexadecimal, "0x10", "Int(16)");
constant!(literal_hexadecimal_upper, "0XFF", "Int(255)");
constant!(
    literal_hexadecimal_above_int_max,
    "0x80000000",
    "UnsignedInt(2147483648)"
);
constant!(literal_octal_above_int_max, "020000000000", "UnsignedInt(2147483648)");
too_large!(literal_above_unsigned_int_max, "0x100000000", "UnsignedLong(0)");
too_large!(literal_above_long_max, "0xffffffffffffffff", "UnsignedLong(4294967295)");

constant!(literal_unsigned_suffix, "1u", "UnsignedInt(1)");
constant!(literal_unsigned_suffix_upper, "1U", "UnsignedInt(1)");
too_large!(literal_unsigned_suffix_promotes, "4294967296u", "UnsignedLong(0)");
constant!(literal_long_suffix, "1l", "Long(1)");
constant!(literal_long_suffix_upper, "1L", "Long(1)");
constant!(literal_unsigned_long_suffix, "1ul", "UnsignedLong(1)");
constant!(literal_long_unsigned_suffix, "1lu", "UnsignedLong(1)");
constant!(literal_unsigned_long_suffix_upper, "1UL", "UnsignedLong(1)");

constant!(literal_double, "1.5", "Double(1.5)");
constant!(literal_double_leading_dot, ".5", "Double(0.5)");
constant!(literal_double_exponent, "1e3", "Double(1000.0)");
constant!(literal_double_negative_exponent, "1e-3", "Double(0.001)");
constant!(literal_float_suffix, "1.5f", "Float(1.5)");
constant!(literal_float_suffix_upper, "1.5F", "Float(1.5)");
constant!(literal_long_double_suffix, "1.5l", "LongDouble(1.5)");

constant!(literal_char, "'a'", "Int(97)");
constant!(literal_char_digit, "'0'", "Int(48)");
constant!(literal_char_escape_newline, "'\\n'", "Int(10)");
constant!(literal_char_escape_tab, "'\\t'", "Int(9)");
constant!(literal_char_escape_null, "'\\0'", "Int(0)");
constant!(literal_char_escape_backslash, "'\\\\'", "Int(92)");
constant!(literal_char_escape_quote, "'\\''", "Int(39)");
constant!(literal_char_escape_octal, "'\\101'", "Int(65)");
constant!(literal_char_escape_hexadecimal, "'\\x41'", "Int(65)");
constant!(literal_char_is_sign_extended, "'\\377'", "Int(-1)");
constant!(literal_char_hexadecimal_is_sign_extended, "'\\xff'", "Int(-1)");
constant!(literal_char_multi_is_packed, "'ab'", "Int(24930)");
constant!(literal_wide_char, "L'a'", "Int(97)");
constant!(literal_wide_char_is_not_sign_extended, "L'\\xff'", "Int(255)");

fold!(
    add_int,
    binary(&ResolvedType::Int, BinaryOp::Add, Value::Int(1), Value::Int(2)),
    "Int(3)"
);
fold_overflow!(
    add_wraps,
    binary(&ResolvedType::Int, BinaryOp::Add, Value::Int(i32::MAX), Value::Int(1)),
    "Int(-2147483648)"
);
fold!(
    sub_unsigned_wraps,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Sub,
        Value::UnsignedInt(1),
        Value::UnsignedInt(2)
    ),
    "UnsignedInt(4294967295)"
);
fold_overflow!(
    mul_wraps,
    binary(&ResolvedType::Int, BinaryOp::Mul, Value::Int(65536), Value::Int(65536)),
    "Int(0)"
);
fold!(
    div_int,
    binary(&ResolvedType::Int, BinaryOp::Div, Value::Int(7), Value::Int(2)),
    "Int(3)"
);
fold!(
    div_negative_truncates_toward_zero,
    binary(&ResolvedType::Int, BinaryOp::Div, Value::Int(-7), Value::Int(2)),
    "Int(-3)"
);
fold!(
    rem_int,
    binary(&ResolvedType::Int, BinaryOp::Mod, Value::Int(7), Value::Int(2)),
    "Int(1)"
);
fold!(
    rem_keeps_sign_of_dividend,
    binary(&ResolvedType::Int, BinaryOp::Mod, Value::Int(-7), Value::Int(2)),
    "Int(-1)"
);

fold!(
    bitand_int,
    binary(&ResolvedType::Int, BinaryOp::BitAnd, Value::Int(6), Value::Int(3)),
    "Int(2)"
);
fold!(
    bitor_int,
    binary(&ResolvedType::Int, BinaryOp::BitOr, Value::Int(6), Value::Int(3)),
    "Int(7)"
);
fold!(
    bitxor_int,
    binary(&ResolvedType::Int, BinaryOp::BitXor, Value::Int(6), Value::Int(3)),
    "Int(5)"
);
fold!(
    shift_left,
    binary(&ResolvedType::Int, BinaryOp::Left, Value::Int(1), Value::Int(4)),
    "Int(16)"
);
fold!(
    shift_left_into_sign_bit,
    binary(&ResolvedType::Int, BinaryOp::Left, Value::Int(1), Value::Int(31)),
    "Int(-2147483648)"
);
fold!(
    shift_right_is_arithmetic,
    binary(&ResolvedType::Int, BinaryOp::Right, Value::Int(-8), Value::Int(1)),
    "Int(-4)"
);
fold!(
    shift_right_unsigned_is_logical,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Right,
        Value::UnsignedInt(2147483648),
        Value::Int(31)
    ),
    "UnsignedInt(1)"
);
fold!(
    shift_keeps_left_operand_type,
    binary(&ResolvedType::Long, BinaryOp::Left, Value::Long(1), Value::Int(1)),
    "Long(2)"
);

fold!(
    neg_int,
    unary(&ResolvedType::Int, UnaryOp::Minus, Value::Int(1)),
    "Int(-1)"
);
fold_overflow!(
    neg_wraps,
    unary(&ResolvedType::Int, UnaryOp::Minus, Value::Int(i32::MIN)),
    "Int(-2147483648)"
);
fold!(
    neg_double,
    unary(&ResolvedType::Double, UnaryOp::Minus, Value::Double(1.5)),
    "Double(-1.5)"
);
fold!(
    bitnot_int,
    unary(&ResolvedType::Int, UnaryOp::BitNot, Value::Int(0)),
    "Int(-1)"
);
fold!(
    bitnot_unsigned,
    unary(&ResolvedType::UnsignedInt, UnaryOp::BitNot, Value::UnsignedInt(0)),
    "UnsignedInt(4294967295)"
);

fold!(
    convert_to_int,
    convert(&ResolvedType::Int, Value::Long(300)),
    "Int(300)"
);
fold!(
    convert_to_int_wraps,
    convert(&ResolvedType::Int, Value::Long(4294967297)),
    "Int(1)"
);
fold!(
    convert_to_unsigned_long_is_32_bits_on_i386,
    convert(&ResolvedType::UnsignedLong, Value::Int(-1)),
    "UnsignedLong(4294967295)"
);
fold!(
    convert_to_long_is_32_bits_on_i386,
    convert(&ResolvedType::Long, Value::UnsignedLong(4294967296)),
    "Long(0)"
);
fold!(
    convert_to_double,
    convert(&ResolvedType::Double, Value::Int(3)),
    "Double(3.0)"
);
fold!(
    convert_from_double_truncates,
    convert(&ResolvedType::Int, Value::Double(3.9)),
    "Int(3)"
);

#[test]
fn a_wider_target_keeps_the_whole_value() {
    let fold = Fold::new(&X86_64);
    assert_eq!(
        repr(fold.convert(&ResolvedType::Long, Value::UnsignedLong(4294967296))),
        "Long(4294967296)"
    );
    assert_eq!(
        repr(fold.convert(&ResolvedType::UnsignedLong, Value::Int(-1))),
        "UnsignedLong(18446744073709551615)"
    );
}

#[test]
fn unsigned_int_and_long_meet_at_long_on_x86_64() {
    let fold = Fold::new(&X86_64);
    assert_eq!(
        repr(fold.binary(
            &ResolvedType::Long,
            BinaryOp::Add,
            Value::UnsignedInt(1),
            Value::Long(-1)
        )),
        "Long(0)"
    );
}

#[test]
fn truncate_carries_the_value_at_full_width() {
    assert_eq!(repr(Value::Int(200).truncate(8, true)), "Long(-56)");
    assert_eq!(repr(Value::Int(200).truncate(8, false)), "Long(200)");
    assert_eq!(repr(Value::Int(-1).truncate(16, false)), "Long(65535)");
    assert_eq!(repr(Value::Int(5).truncate(64, true)), "Long(5)");
    assert_eq!(
        repr(Value::Int(-1).truncate(64, false)),
        "UnsignedLong(18446744073709551615)"
    );
    assert_eq!(
        repr(Value::Int(-1).truncate(0, false)),
        "UnsignedLong(18446744073709551615)"
    );
}

#[test]
fn logical_not_tests_against_zero() {
    assert_eq!(repr(Value::Int(0).logical_not()), "Int(1)");
    assert_eq!(repr(Value::Int(42).logical_not()), "Int(0)");
    assert_eq!(repr(Value::Double(0.0).logical_not()), "Int(1)");
    assert_eq!(repr(Value::from(true)), "Int(1)");
    assert_eq!(repr(Value::from(false)), "Int(0)");
}

#[test]
fn is_true_follows_zero_test() {
    assert!(Value::Int(1).is_true());
    assert!(!Value::Int(0).is_true());
    assert!(Value::Double(0.5).is_true());
    assert!(!Value::Double(0.0).is_true());
    assert!(!Value::UnsignedLong(0).is_true());
}

#[test]
fn is_floating_covers_real_types() {
    assert!(Value::Float(0.0).is_floating());
    assert!(Value::Double(0.0).is_floating());
    assert!(Value::LongDouble(0.0).is_floating());
    assert!(!Value::Int(0).is_floating());
    assert!(!Value::UnsignedLong(0).is_floating());
}

#[test]
fn get_integer_value_rejects_real_types() {
    assert_eq!(Value::Int(1).get_integer_value(), Some(1));
    assert_eq!(Value::Int(-1).get_integer_value(), Some(u64::MAX));
    assert_eq!(Value::UnsignedLong(u64::MAX).get_integer_value(), Some(u64::MAX));
    assert_eq!(Value::Double(1.0).get_integer_value(), None);
    assert_eq!(Value::Float(1.0).get_integer_value(), None);
    assert_eq!(Value::LongDouble(1.0).get_integer_value(), None);
}

#[test]
fn to_i64_and_to_u64_reinterpret() {
    assert_eq!(Value::Int(-1).to_i64(), -1);
    assert_eq!(Value::Int(-1).to_u64(), u64::MAX);
    assert_eq!(Value::UnsignedInt(u32::MAX).to_i64(), u32::MAX as i64);
    assert_eq!(Value::Double(3.9).to_i64(), 3);
    assert_eq!(Value::Double(-3.9).to_i64(), -3);
}

#[test]
fn equality_compares_same_type_operands() {
    let fold = Fold::new(&I386);
    assert!(matches!(
        fold.compare(Value::Int(1), Value::Int(1)),
        Some(Ordering::Equal)
    ));
    assert!(matches!(
        fold.compare(Value::Double(1.0), Value::Double(1.0)),
        Some(Ordering::Equal)
    ));
    assert!(matches!(
        fold.compare(Value::Int(1), Value::Int(2)),
        Some(Ordering::Less | Ordering::Greater)
    ));
}

#[test]
fn ordering_compares_same_type_operands() {
    let fold = Fold::new(&I386);
    assert!(matches!(
        fold.compare(Value::Int(1), Value::Int(2)),
        Some(Ordering::Less)
    ));
    assert!(matches!(
        fold.compare(Value::Double(1.0), Value::Double(1.5)),
        Some(Ordering::Less)
    ));
    assert!(matches!(
        fold.compare(Value::UnsignedInt(1), Value::UnsignedInt(0)),
        Some(Ordering::Greater)
    ));
    assert!(matches!(
        fold.compare(Value::Long(-1), Value::Long(0)),
        Some(Ordering::Less)
    ));
    assert!(matches!(
        fold.compare(Value::Int(2), Value::Int(2)),
        Some(Ordering::Less | Ordering::Equal)
    ));
    assert!(matches!(
        fold.compare(Value::Int(2), Value::Int(2)),
        Some(Ordering::Greater | Ordering::Equal)
    ));
}

#[test]
fn every_operator_folds_in_sequence() {
    let fold = Fold::new(&I386);
    let ty = ResolvedType::Int;
    let steps = [
        (BinaryOp::Add, 2, "Int(3)"),
        (BinaryOp::Mul, 4, "Int(12)"),
        (BinaryOp::Sub, 2, "Int(10)"),
        (BinaryOp::Div, 3, "Int(3)"),
        (BinaryOp::Mod, 2, "Int(1)"),
        (BinaryOp::Left, 3, "Int(8)"),
        (BinaryOp::Right, 1, "Int(4)"),
        (BinaryOp::BitOr, 1, "Int(5)"),
        (BinaryOp::BitAnd, 3, "Int(1)"),
        (BinaryOp::BitXor, 3, "Int(2)"),
    ];
    let mut value = Value::Int(1);
    for (op, rhs, expected) in steps {
        value = fold.binary(&ty, op, value, Value::Int(rhs)).res;
        assert_eq!(repr(value), expected, "{op:?}");
    }
}

#[test]
fn the_minimum_of_a_signed_type_is_recognised() {
    let fold = Fold::new(&I386);
    assert!(fold.is_min(&ResolvedType::Int, Value::Int(i32::MIN)));
    assert!(fold.is_min(&ResolvedType::Long, Value::Long(-2147483648)));
    assert!(!fold.is_min(&ResolvedType::Int, Value::Int(0)));
    assert!(!fold.is_min(&ResolvedType::UnsignedInt, Value::UnsignedInt(0)));
    assert!(!fold.is_min(&ResolvedType::UnsignedLong, Value::UnsignedLong(0)));
}
