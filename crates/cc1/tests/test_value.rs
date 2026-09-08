use std::cmp::Ordering;

use cc1::ast::{BinaryOp, F80, Fold, UnaryOp, Value};
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
constant!(literal_char_escape_bell, "'\\a'", "Int(7)");
constant!(literal_char_escape_backspace, "'\\b'", "Int(8)");
constant!(literal_char_escape_form_feed, "'\\f'", "Int(12)");
constant!(literal_char_escape_carriage_return, "'\\r'", "Int(13)");
constant!(literal_char_escape_vertical_tab, "'\\v'", "Int(11)");
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
    assert!(Value::LongDouble(F80::from(0.0)).is_floating());
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
    assert_eq!(Value::LongDouble(F80::from(1.0)).get_integer_value(), None);
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

fold!(
    unsigned_div,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Div,
        Value::UnsignedInt(7),
        Value::UnsignedInt(2)
    ),
    "UnsignedInt(3)"
);
fold!(
    unsigned_rem,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Mod,
        Value::UnsignedInt(7),
        Value::UnsignedInt(2)
    ),
    "UnsignedInt(1)"
);
fold!(
    unsigned_mul_wraps_without_a_diagnosis,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Mul,
        Value::UnsignedInt(65536),
        Value::UnsignedInt(65536)
    ),
    "UnsignedInt(0)"
);
fold!(
    unsigned_bitor,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::BitOr,
        Value::UnsignedInt(4026531840),
        Value::UnsignedInt(15)
    ),
    "UnsignedInt(4026531855)"
);
fold!(
    unsigned_long_sub_wraps,
    binary(
        &ResolvedType::UnsignedLong,
        BinaryOp::Sub,
        Value::UnsignedLong(0),
        Value::UnsignedLong(1)
    ),
    "UnsignedLong(4294967295)"
);

// 6.2.1.5 the width of the result is the target's: long is 32 bits on i386.
fold_overflow!(
    long_add_wraps_at_the_width_of_the_target,
    binary(
        &ResolvedType::Long,
        BinaryOp::Add,
        Value::Long(2147483647),
        Value::Long(1)
    ),
    "Long(-2147483648)"
);

#[test]
fn the_same_addition_fits_a_64_bit_long() {
    let fold = Fold::new(&X86_64);
    let folded = fold.binary(
        &ResolvedType::Long,
        BinaryOp::Add,
        Value::Long(2147483647),
        Value::Long(1),
    );
    assert_eq!(repr(folded.res), "Long(2147483648)");
    assert!(folded.diagnosis.is_none(), "{:?}", folded.diagnosis);
}

fold!(
    double_sub,
    binary(
        &ResolvedType::Double,
        BinaryOp::Sub,
        Value::Double(1.5),
        Value::Double(0.25)
    ),
    "Double(1.25)"
);
fold!(
    double_mul,
    binary(
        &ResolvedType::Double,
        BinaryOp::Mul,
        Value::Double(1.5),
        Value::Double(2.0)
    ),
    "Double(3.0)"
);
fold!(
    long_double_add,
    binary(
        &ResolvedType::LongDouble,
        BinaryOp::Add,
        Value::LongDouble(F80::from(1.5)),
        Value::LongDouble(F80::from(1.5))
    ),
    "LongDouble(3.0)"
);
// 6.2.1.4 a float result carries only the precision of a float: 16777216 + 1 is not representable.
fold!(
    float_arithmetic_rounds_to_float_precision,
    binary(
        &ResolvedType::Float,
        BinaryOp::Add,
        Value::Float(16777216.0),
        Value::Float(1.0)
    ),
    "Float(16777216.0)"
);

fold!(
    neg_unsigned_wraps,
    unary(&ResolvedType::UnsignedInt, UnaryOp::Minus, Value::UnsignedInt(1)),
    "UnsignedInt(4294967295)"
);
fold!(
    neg_long,
    unary(&ResolvedType::Long, UnaryOp::Minus, Value::Long(-1)),
    "Long(1)"
);
// The negation of a floating zero is a negative zero.
fold!(
    neg_floating_zero_keeps_its_sign,
    unary(&ResolvedType::Double, UnaryOp::Minus, Value::Double(0.0)),
    "Double(-0.0)"
);
fold!(
    bitnot_long,
    unary(&ResolvedType::Long, UnaryOp::BitNot, Value::Long(0)),
    "Long(-1)"
);

fold!(
    convert_to_a_narrow_integer_type,
    convert(&ResolvedType::Char, Value::Int(300)),
    "Int(44)"
);
fold!(
    convert_to_unsigned_int_wraps,
    convert(&ResolvedType::UnsignedInt, Value::Int(-1)),
    "UnsignedInt(4294967295)"
);
fold!(
    convert_to_long_double,
    convert(&ResolvedType::LongDouble, Value::Int(3)),
    "LongDouble(3.0)"
);
// 6.2.1.4 a value converted to float takes the nearest representable value.
fold!(
    convert_to_float_rounds,
    convert(&ResolvedType::Float, Value::Double(16777217.0)),
    "Float(16777216.0)"
);

#[test]
fn convert_rejects_a_type_that_holds_no_value() {
    let fold = Fold::new(&I386);
    assert_eq!(fold.convert(&ResolvedType::Void, Value::Int(1)), None);
}

// 6.2.1.5 the operands of a comparison reach the fold already converted to a common type.
#[test]
fn comparing_unconverted_operands_yields_no_ordering() {
    let fold = Fold::new(&I386);
    assert_eq!(fold.compare(Value::Int(1), Value::Double(1.0)), None);
}

// 6.3 only the additive and multiplicative operators can leave the range of their type; a shift,
// a division or a bitwise operator never reports an overflow.
#[test]
fn only_additive_and_multiplicative_results_report_an_overflow() {
    let fold = Fold::new(&I386);
    for (op, lhs, rhs) in [
        (BinaryOp::Div, Value::Int(i32::MIN), Value::Int(-1)),
        (BinaryOp::Left, Value::Int(1), Value::Int(31)),
        (BinaryOp::BitXor, Value::Int(-1), Value::Int(0)),
    ] {
        let folded = fold.binary(&ResolvedType::Int, op, lhs, rhs);
        assert!(folded.diagnosis.is_none(), "{op:?} reported {:?}", folded.diagnosis);
    }
}

fold!(
    unsigned_int_shift_left,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Left,
        Value::UnsignedInt(1),
        Value::Int(31)
    ),
    "UnsignedInt(2147483648)"
);
fold!(
    unsigned_long_shift_left,
    binary(
        &ResolvedType::UnsignedLong,
        BinaryOp::Left,
        Value::UnsignedLong(1),
        Value::Int(4)
    ),
    "UnsignedLong(16)"
);
fold!(
    long_shift_right_is_arithmetic,
    binary(&ResolvedType::Long, BinaryOp::Right, Value::Long(-8), Value::Int(1)),
    "Long(-4)"
);
fold!(
    unsigned_long_shift_right_is_logical,
    binary(
        &ResolvedType::UnsignedLong,
        BinaryOp::Right,
        Value::UnsignedLong(2147483648),
        Value::Int(31)
    ),
    "UnsignedLong(1)"
);

// 6.3.7 The type of the result is that of the promoted left operand: a long is 32 bits on i386,
// so shifting into its sign bit yields a negative long.
// gcc: `enum E { A = 1L << 31 };` is accepted on i386, where 1L << 31 is -2147483648.
fold!(
    a_shift_narrows_to_the_width_of_its_type,
    binary(&ResolvedType::Long, BinaryOp::Left, Value::Long(1), Value::Int(31)),
    "Long(-2147483648)"
);

#[test]
fn the_same_shift_keeps_its_value_in_a_64_bit_long() {
    let fold = Fold::new(&X86_64);
    let shifted = fold.binary(&ResolvedType::Long, BinaryOp::Left, Value::Long(1), Value::Int(31));
    assert_eq!(repr(shifted.res), "Long(2147483648)");
}

// 6.3.3.3 The result of the ~ operator is the bitwise complement of its promoted operand, taken
// at the width the target gives that type.
fold!(
    bitnot_unsigned_long,
    unary(&ResolvedType::UnsignedLong, UnaryOp::BitNot, Value::UnsignedLong(0)),
    "UnsignedLong(4294967295)"
);

#[test]
fn the_same_complement_fills_a_64_bit_unsigned_long() {
    let fold = Fold::new(&X86_64);
    let value = fold.unary(&ResolvedType::UnsignedLong, UnaryOp::BitNot, Value::UnsignedLong(0));
    assert_eq!(repr(value.res), "UnsignedLong(18446744073709551615)");
}

fold!(
    double_div,
    binary(
        &ResolvedType::Double,
        BinaryOp::Div,
        Value::Double(3.0),
        Value::Double(2.0)
    ),
    "Double(1.5)"
);
fold!(
    unsigned_add_wraps,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::Add,
        Value::UnsignedInt(4294967295),
        Value::UnsignedInt(1)
    ),
    "UnsignedInt(0)"
);
fold!(
    unsigned_bitxor,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::BitXor,
        Value::UnsignedInt(6),
        Value::UnsignedInt(3)
    ),
    "UnsignedInt(5)"
);

fold!(
    convert_an_unsigned_int_to_double,
    convert(&ResolvedType::Double, Value::UnsignedInt(1)),
    "Double(1.0)"
);
fold!(
    convert_a_long_to_double,
    convert(&ResolvedType::Double, Value::Long(2)),
    "Double(2.0)"
);
fold!(
    convert_an_unsigned_long_to_double,
    convert(&ResolvedType::Double, Value::UnsignedLong(3)),
    "Double(3.0)"
);

#[test]
fn compare_orders_every_representation() {
    let fold = Fold::new(&I386);
    assert_eq!(
        fold.compare(Value::UnsignedLong(1), Value::UnsignedLong(2)),
        Some(Ordering::Less)
    );
    assert_eq!(fold.compare(Value::Float(1.0), Value::Float(2.0)), Some(Ordering::Less));
    assert_eq!(
        fold.compare(Value::LongDouble(F80::from(2.0)), Value::LongDouble(F80::from(2.0))),
        Some(Ordering::Equal)
    );
    assert_eq!(fold.compare(Value::Double(f64::NAN), Value::Double(1.0)), None);
}

// 6.3.7 The right operand of a shift shall be nonnegative: the check reaches every representation
// a folded count can have.
#[test]
fn is_negative_covers_every_representation() {
    assert!(Value::Int(-1).is_negative());
    assert!(Value::Long(-1).is_negative());
    assert!(Value::Float(-1.0).is_negative());
    assert!(Value::Double(-1.0).is_negative());
    assert!(Value::LongDouble(F80::from(-1.0)).is_negative());
    assert!(!Value::Int(1).is_negative());
    assert!(!Value::UnsignedInt(1).is_negative());
    assert!(!Value::UnsignedLong(1).is_negative());
}

#[test]
fn is_greater_or_eq_covers_every_representation() {
    assert!(Value::Int(5).is_greater_or_eq(4));
    assert!(Value::UnsignedInt(5).is_greater_or_eq(4));
    assert!(Value::UnsignedLong(4).is_greater_or_eq(4));
    assert!(Value::Float(4.5).is_greater_or_eq(4));
    assert!(Value::LongDouble(F80::from(4.0)).is_greater_or_eq(4));
    assert!(!Value::Long(3).is_greater_or_eq(4));
    assert!(!Value::Double(3.5).is_greater_or_eq(4));
}

#[test]
fn is_zero_covers_every_representation() {
    assert!(Value::Float(0.0).is_zero());
    assert!(Value::Double(0.0).is_zero());
    assert!(Value::LongDouble(F80::from(0.0)).is_zero());
    assert!(!Value::Float(1.0).is_zero());
    assert!(!Value::LongDouble(F80::from(1.0)).is_zero());
}

#[test]
fn a_floating_value_reinterprets_as_an_integer_by_truncation() {
    assert_eq!(Value::Float(3.9).to_i64(), 3);
    assert_eq!(Value::Float(3.9).to_u64(), 3);
    assert_eq!(Value::Double(3.9).to_u64(), 3);
    assert_eq!(Value::LongDouble(F80::from(3.9)).to_u64(), 3);
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
