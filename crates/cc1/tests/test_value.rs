use std::cmp::Ordering;

use cc1::ast::{BinaryOp, ConstFolder, ConstValue, F80, UnaryOp};
use cc1::semantic::{Diag, Diagnostic, ResolvedType};

trait FoldValue {
    fn fold_value(self) -> ConstValue;
}

impl FoldValue for ConstValue {
    fn fold_value(self) -> ConstValue {
        self
    }
}

impl FoldValue for Diag<ConstValue> {
    fn fold_value(self) -> ConstValue {
        self.res
    }
}

impl FoldValue for Option<ConstValue> {
    fn fold_value(self) -> ConstValue {
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
            let fold = ConstFolder;
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
            let fold = ConstFolder;
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
            let Diag { res, diagnostic } = ConstFolder.$method($($arg),*);
            assert_eq!(repr(res), $expected, "{}", stringify!($method($($arg),*)));
            assert!(
                matches!(diagnostic, Some(Diagnostic::ArithmeticOverflow)),
                "{} should report ArithmeticOverflow, got {diagnostic:?}",
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
constant!(literal_hexadecimal_above_int_max, "0x80000000", "UnsignedInt(2147483648)");
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

// 6.1.3.4 The value of an octal or hexadecimal escape sequence shall be in the range of
// representable values for the type unsigned char for an integer character constant, or the
// unsigned type corresponding to wchar_t for a wide character constant.
escape_out_of_range!(literal_char_octal_escape_out_of_range, "'\\777'", "Int(-1)");
escape_out_of_range!(literal_char_octal_escape_just_out_of_range, "'\\400'", "Int(0)");
escape_out_of_range!(literal_char_hex_escape_out_of_range, "'\\x100'", "Int(0)");
escape_out_of_range!(literal_char_hex_escape_far_out_of_range, "'\\x1ff'", "Int(-1)");
escape_out_of_range!(literal_multi_char_escape_out_of_range, "'a\\x100'", "Int(24832)");
constant!(literal_char_octal_escape_at_range_limit, "'\\377'", "Int(-1)");
constant!(literal_char_hex_escape_at_range_limit, "'\\xff'", "Int(-1)");
constant!(literal_wide_char_hex_escape_above_char_range, "L'\\x100'", "Int(256)");
constant!(literal_wide_char_hex_escape_uses_wchar_range, "L'\\xffff'", "Int(65535)");

fold!(add_int, binary(&ResolvedType::Int, BinaryOp::Add, ConstValue::Int(1), ConstValue::Int(2)), "Int(3)");
fold_overflow!(
    add_wraps,
    binary(&ResolvedType::Int, BinaryOp::Add, ConstValue::Int(i32::MAX), ConstValue::Int(1)),
    "Int(-2147483648)"
);
fold!(
    sub_unsigned_wraps,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Sub, ConstValue::UnsignedInt(1), ConstValue::UnsignedInt(2)),
    "UnsignedInt(4294967295)"
);
fold_overflow!(
    mul_wraps,
    binary(&ResolvedType::Int, BinaryOp::Mul, ConstValue::Int(65536), ConstValue::Int(65536)),
    "Int(0)"
);
fold!(div_int, binary(&ResolvedType::Int, BinaryOp::Div, ConstValue::Int(7), ConstValue::Int(2)), "Int(3)");
fold!(
    div_negative_truncates_toward_zero,
    binary(&ResolvedType::Int, BinaryOp::Div, ConstValue::Int(-7), ConstValue::Int(2)),
    "Int(-3)"
);
fold!(rem_int, binary(&ResolvedType::Int, BinaryOp::Mod, ConstValue::Int(7), ConstValue::Int(2)), "Int(1)");
fold!(
    rem_keeps_sign_of_dividend,
    binary(&ResolvedType::Int, BinaryOp::Mod, ConstValue::Int(-7), ConstValue::Int(2)),
    "Int(-1)"
);

fold!(bitand_int, binary(&ResolvedType::Int, BinaryOp::BitAnd, ConstValue::Int(6), ConstValue::Int(3)), "Int(2)");
fold!(bitor_int, binary(&ResolvedType::Int, BinaryOp::BitOr, ConstValue::Int(6), ConstValue::Int(3)), "Int(7)");
fold!(bitxor_int, binary(&ResolvedType::Int, BinaryOp::BitXor, ConstValue::Int(6), ConstValue::Int(3)), "Int(5)");
fold!(shift_left, binary(&ResolvedType::Int, BinaryOp::Left, ConstValue::Int(1), ConstValue::Int(4)), "Int(16)");
fold!(
    shift_left_into_sign_bit,
    binary(&ResolvedType::Int, BinaryOp::Left, ConstValue::Int(1), ConstValue::Int(31)),
    "Int(-2147483648)"
);
fold!(
    shift_right_is_arithmetic,
    binary(&ResolvedType::Int, BinaryOp::Right, ConstValue::Int(-8), ConstValue::Int(1)),
    "Int(-4)"
);
fold!(
    shift_right_unsigned_is_logical,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Right, ConstValue::UnsignedInt(2147483648), ConstValue::Int(31)),
    "UnsignedInt(1)"
);
fold!(
    shift_keeps_left_operand_type,
    binary(&ResolvedType::Long, BinaryOp::Left, ConstValue::Long(1), ConstValue::Int(1)),
    "Long(2)"
);

fold!(neg_int, unary(&ResolvedType::Int, UnaryOp::Minus, ConstValue::Int(1)), "Int(-1)");
fold_overflow!(neg_wraps, unary(&ResolvedType::Int, UnaryOp::Minus, ConstValue::Int(i32::MIN)), "Int(-2147483648)");
fold!(neg_double, unary(&ResolvedType::Double, UnaryOp::Minus, ConstValue::Double(1.5)), "Double(-1.5)");
fold!(bitnot_int, unary(&ResolvedType::Int, UnaryOp::BitNot, ConstValue::Int(0)), "Int(-1)");
fold!(
    bitnot_unsigned,
    unary(&ResolvedType::UnsignedInt, UnaryOp::BitNot, ConstValue::UnsignedInt(0)),
    "UnsignedInt(4294967295)"
);

fold!(convert_to_int, convert(&ResolvedType::Int, ConstValue::Long(300)), "Int(300)");
fold!(convert_to_int_wraps, convert(&ResolvedType::Int, ConstValue::Long(4294967297)), "Int(1)");
fold!(
    convert_to_unsigned_long_is_32_bits_on_i386,
    convert(&ResolvedType::UnsignedLong, ConstValue::Int(-1)),
    "UnsignedLong(4294967295)"
);
fold!(
    convert_to_long_is_32_bits_on_i386,
    convert(&ResolvedType::Long, ConstValue::UnsignedLong(4294967296)),
    "Long(0)"
);
fold!(convert_to_double, convert(&ResolvedType::Double, ConstValue::Int(3)), "Double(3.0)");
fold!(convert_from_double_truncates, convert(&ResolvedType::Int, ConstValue::Double(3.9)), "Int(3)");

#[test]
fn logical_not_tests_against_zero() {
    assert_eq!(repr(ConstValue::Int(0).logical_not()), "Int(1)");
    assert_eq!(repr(ConstValue::Int(42).logical_not()), "Int(0)");
    assert_eq!(repr(ConstValue::Double(0.0).logical_not()), "Int(1)");
    assert_eq!(repr(ConstValue::from(true)), "Int(1)");
    assert_eq!(repr(ConstValue::from(false)), "Int(0)");
}

#[test]
fn is_true_follows_zero_test() {
    assert!(ConstValue::Int(1).is_true());
    assert!(!ConstValue::Int(0).is_true());
    assert!(ConstValue::Double(0.5).is_true());
    assert!(!ConstValue::Double(0.0).is_true());
    assert!(!ConstValue::UnsignedLong(0).is_true());
}

#[test]
fn is_floating_covers_real_types() {
    assert!(ConstValue::Float(0.0).is_floating());
    assert!(ConstValue::Double(0.0).is_floating());
    assert!(ConstValue::LongDouble(F80::from(0.0)).is_floating());
    assert!(!ConstValue::Int(0).is_floating());
    assert!(!ConstValue::UnsignedLong(0).is_floating());
}

#[test]
fn get_integer_value_rejects_real_types() {
    assert_eq!(ConstValue::Int(1).get_integer_value(), Some(1));
    assert_eq!(ConstValue::Int(-1).get_integer_value(), Some(u64::MAX));
    assert_eq!(ConstValue::UnsignedLong(u64::MAX).get_integer_value(), Some(u64::MAX));
    assert_eq!(ConstValue::Double(1.0).get_integer_value(), None);
    assert_eq!(ConstValue::Float(1.0).get_integer_value(), None);
    assert_eq!(ConstValue::LongDouble(F80::from(1.0)).get_integer_value(), None);
}

#[test]
fn to_i64_and_to_u64_reinterpret() {
    assert_eq!(ConstValue::Int(-1).to_i64(), -1);
    assert_eq!(ConstValue::Int(-1).to_u64(), u64::MAX);
    assert_eq!(ConstValue::UnsignedInt(u32::MAX).to_i64(), u32::MAX as i64);
    assert_eq!(ConstValue::Double(3.9).to_i64(), 3);
    assert_eq!(ConstValue::Double(-3.9).to_i64(), -3);
}

#[test]
fn equality_compares_same_type_operands() {
    let fold = ConstFolder;
    assert!(matches!(fold.compare(ConstValue::Int(1), ConstValue::Int(1)), Some(Ordering::Equal)));
    assert!(matches!(fold.compare(ConstValue::Double(1.0), ConstValue::Double(1.0)), Some(Ordering::Equal)));
    assert!(matches!(fold.compare(ConstValue::Int(1), ConstValue::Int(2)), Some(Ordering::Less | Ordering::Greater)));
}

#[test]
fn ordering_compares_same_type_operands() {
    let fold = ConstFolder;
    assert!(matches!(fold.compare(ConstValue::Int(1), ConstValue::Int(2)), Some(Ordering::Less)));
    assert!(matches!(fold.compare(ConstValue::Double(1.0), ConstValue::Double(1.5)), Some(Ordering::Less)));
    assert!(matches!(fold.compare(ConstValue::UnsignedInt(1), ConstValue::UnsignedInt(0)), Some(Ordering::Greater)));
    assert!(matches!(fold.compare(ConstValue::Long(-1), ConstValue::Long(0)), Some(Ordering::Less)));
    assert!(matches!(fold.compare(ConstValue::Int(2), ConstValue::Int(2)), Some(Ordering::Less | Ordering::Equal)));
    assert!(matches!(fold.compare(ConstValue::Int(2), ConstValue::Int(2)), Some(Ordering::Greater | Ordering::Equal)));
}

#[test]
fn every_operator_folds_in_sequence() {
    let fold = ConstFolder;
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
    let mut value = ConstValue::Int(1);
    for (op, rhs, expected) in steps {
        value = fold.binary(&ty, op, value, ConstValue::Int(rhs)).res;
        assert_eq!(repr(value), expected, "{op:?}");
    }
}

fold!(
    unsigned_div,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Div, ConstValue::UnsignedInt(7), ConstValue::UnsignedInt(2)),
    "UnsignedInt(3)"
);
fold!(
    unsigned_rem,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Mod, ConstValue::UnsignedInt(7), ConstValue::UnsignedInt(2)),
    "UnsignedInt(1)"
);
fold!(
    unsigned_mul_wraps_without_a_diagnostic,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Mul, ConstValue::UnsignedInt(65536), ConstValue::UnsignedInt(65536)),
    "UnsignedInt(0)"
);
fold!(
    unsigned_bitor,
    binary(
        &ResolvedType::UnsignedInt,
        BinaryOp::BitOr,
        ConstValue::UnsignedInt(4026531840),
        ConstValue::UnsignedInt(15)
    ),
    "UnsignedInt(4026531855)"
);
fold!(
    unsigned_long_sub_wraps,
    binary(&ResolvedType::UnsignedLong, BinaryOp::Sub, ConstValue::UnsignedLong(0), ConstValue::UnsignedLong(1)),
    "UnsignedLong(4294967295)"
);

// 6.2.1.5 the width of the result is the target's: long is 32 bits on i386.
fold_overflow!(
    long_add_wraps_at_32_bits,
    binary(&ResolvedType::Long, BinaryOp::Add, ConstValue::Long(2147483647), ConstValue::Long(1)),
    "Long(-2147483648)"
);

fold!(
    double_sub,
    binary(&ResolvedType::Double, BinaryOp::Sub, ConstValue::Double(1.5), ConstValue::Double(0.25)),
    "Double(1.25)"
);
fold!(
    double_mul,
    binary(&ResolvedType::Double, BinaryOp::Mul, ConstValue::Double(1.5), ConstValue::Double(2.0)),
    "Double(3.0)"
);
fold!(
    long_double_add,
    binary(
        &ResolvedType::LongDouble,
        BinaryOp::Add,
        ConstValue::LongDouble(F80::from(1.5)),
        ConstValue::LongDouble(F80::from(1.5))
    ),
    "LongDouble(3.0)"
);
// 6.2.1.4 a float result carries only the precision of a float: 16777216 + 1 is not representable.
fold!(
    float_arithmetic_rounds_to_float_precision,
    binary(&ResolvedType::Float, BinaryOp::Add, ConstValue::Float(16777216.0), ConstValue::Float(1.0)),
    "Float(16777216.0)"
);

fold!(
    neg_unsigned_wraps,
    unary(&ResolvedType::UnsignedInt, UnaryOp::Minus, ConstValue::UnsignedInt(1)),
    "UnsignedInt(4294967295)"
);
fold!(neg_long, unary(&ResolvedType::Long, UnaryOp::Minus, ConstValue::Long(-1)), "Long(1)");
// The negation of a floating zero is a negative zero.
fold!(
    neg_floating_zero_keeps_its_sign,
    unary(&ResolvedType::Double, UnaryOp::Minus, ConstValue::Double(0.0)),
    "Double(-0.0)"
);
fold!(bitnot_long, unary(&ResolvedType::Long, UnaryOp::BitNot, ConstValue::Long(0)), "Long(-1)");

fold!(convert_to_a_narrow_integer_type, convert(&ResolvedType::Char, ConstValue::Int(300)), "Int(44)");
fold!(
    convert_to_unsigned_int_wraps,
    convert(&ResolvedType::UnsignedInt, ConstValue::Int(-1)),
    "UnsignedInt(4294967295)"
);
fold!(convert_to_long_double, convert(&ResolvedType::LongDouble, ConstValue::Int(3)), "LongDouble(3.0)");
// 6.2.1.4 a value converted to float takes the nearest representable value.
fold!(convert_to_float_rounds, convert(&ResolvedType::Float, ConstValue::Double(16777217.0)), "Float(16777216.0)");

#[test]
fn convert_rejects_a_type_that_holds_no_value() {
    let fold = ConstFolder;
    assert_eq!(fold.convert(&ResolvedType::Void, ConstValue::Int(1)), None);
}

// 6.2.1.5 the operands of a comparison reach the fold already converted to a common type.
#[test]
fn comparing_unconverted_operands_yields_no_ordering() {
    let fold = ConstFolder;
    assert_eq!(fold.compare(ConstValue::Int(1), ConstValue::Double(1.0)), None);
}

// 6.3 only the additive and multiplicative operators can leave the range of their type; a shift,
// a division or a bitwise operator never reports an overflow.
#[test]
fn only_additive_and_multiplicative_results_report_an_overflow() {
    let fold = ConstFolder;
    for (op, lhs, rhs) in [
        (BinaryOp::Div, ConstValue::Int(i32::MIN), ConstValue::Int(-1)),
        (BinaryOp::Left, ConstValue::Int(1), ConstValue::Int(31)),
        (BinaryOp::BitXor, ConstValue::Int(-1), ConstValue::Int(0)),
    ] {
        let folded = fold.binary(&ResolvedType::Int, op, lhs, rhs);
        assert!(folded.diagnostic.is_none(), "{op:?} reported {:?}", folded.diagnostic);
    }
}

fold!(
    unsigned_int_shift_left,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Left, ConstValue::UnsignedInt(1), ConstValue::Int(31)),
    "UnsignedInt(2147483648)"
);
fold!(
    unsigned_long_shift_left,
    binary(&ResolvedType::UnsignedLong, BinaryOp::Left, ConstValue::UnsignedLong(1), ConstValue::Int(4)),
    "UnsignedLong(16)"
);
fold!(
    long_shift_right_is_arithmetic,
    binary(&ResolvedType::Long, BinaryOp::Right, ConstValue::Long(-8), ConstValue::Int(1)),
    "Long(-4)"
);
fold!(
    unsigned_long_shift_right_is_logical,
    binary(&ResolvedType::UnsignedLong, BinaryOp::Right, ConstValue::UnsignedLong(2147483648), ConstValue::Int(31)),
    "UnsignedLong(1)"
);

// 6.3.7 The type of the result is that of the promoted left operand: a long is 32 bits on i386,
// so shifting into its sign bit yields a negative long.
// gcc: `enum E { A = 1L << 31 };` is accepted on i386, where 1L << 31 is -2147483648.
fold!(
    a_shift_narrows_to_the_width_of_its_type,
    binary(&ResolvedType::Long, BinaryOp::Left, ConstValue::Long(1), ConstValue::Int(31)),
    "Long(-2147483648)"
);

// 6.3.3.3 The result of the ~ operator is the bitwise complement of its promoted operand, taken
// at the width the target gives that type.
fold!(
    bitnot_unsigned_long,
    unary(&ResolvedType::UnsignedLong, UnaryOp::BitNot, ConstValue::UnsignedLong(0)),
    "UnsignedLong(4294967295)"
);

fold!(
    double_div,
    binary(&ResolvedType::Double, BinaryOp::Div, ConstValue::Double(3.0), ConstValue::Double(2.0)),
    "Double(1.5)"
);
fold!(
    unsigned_add_wraps,
    binary(&ResolvedType::UnsignedInt, BinaryOp::Add, ConstValue::UnsignedInt(4294967295), ConstValue::UnsignedInt(1)),
    "UnsignedInt(0)"
);
fold!(
    unsigned_bitxor,
    binary(&ResolvedType::UnsignedInt, BinaryOp::BitXor, ConstValue::UnsignedInt(6), ConstValue::UnsignedInt(3)),
    "UnsignedInt(5)"
);

fold!(convert_an_unsigned_int_to_double, convert(&ResolvedType::Double, ConstValue::UnsignedInt(1)), "Double(1.0)");
fold!(convert_a_long_to_double, convert(&ResolvedType::Double, ConstValue::Long(2)), "Double(2.0)");
fold!(convert_an_unsigned_long_to_double, convert(&ResolvedType::Double, ConstValue::UnsignedLong(3)), "Double(3.0)");

#[test]
fn compare_orders_every_representation() {
    let fold = ConstFolder;
    assert_eq!(fold.compare(ConstValue::UnsignedLong(1), ConstValue::UnsignedLong(2)), Some(Ordering::Less));
    assert_eq!(fold.compare(ConstValue::Float(1.0), ConstValue::Float(2.0)), Some(Ordering::Less));
    assert_eq!(
        fold.compare(ConstValue::LongDouble(F80::from(2.0)), ConstValue::LongDouble(F80::from(2.0))),
        Some(Ordering::Equal)
    );
    assert_eq!(fold.compare(ConstValue::Double(f64::NAN), ConstValue::Double(1.0)), None);
}

// 6.3.7 The right operand of a shift shall be nonnegative: the check reaches every representation
// a folded count can have.
#[test]
fn is_negative_covers_every_representation() {
    assert!(ConstValue::Int(-1).is_negative());
    assert!(ConstValue::Long(-1).is_negative());
    assert!(ConstValue::Float(-1.0).is_negative());
    assert!(ConstValue::Double(-1.0).is_negative());
    assert!(ConstValue::LongDouble(F80::from(-1.0)).is_negative());
    assert!(!ConstValue::Int(1).is_negative());
    assert!(!ConstValue::UnsignedInt(1).is_negative());
    assert!(!ConstValue::UnsignedLong(1).is_negative());
}

#[test]
fn is_greater_or_eq_covers_every_representation() {
    assert!(ConstValue::Int(5).is_greater_or_eq(4));
    assert!(ConstValue::UnsignedInt(5).is_greater_or_eq(4));
    assert!(ConstValue::UnsignedLong(4).is_greater_or_eq(4));
    assert!(ConstValue::Float(4.5).is_greater_or_eq(4));
    assert!(ConstValue::LongDouble(F80::from(4.0)).is_greater_or_eq(4));
    assert!(!ConstValue::Long(3).is_greater_or_eq(4));
    assert!(!ConstValue::Double(3.5).is_greater_or_eq(4));
}

#[test]
fn is_zero_covers_every_representation() {
    assert!(ConstValue::Float(0.0).is_zero());
    assert!(ConstValue::Double(0.0).is_zero());
    assert!(ConstValue::LongDouble(F80::from(0.0)).is_zero());
    assert!(!ConstValue::Float(1.0).is_zero());
    assert!(!ConstValue::LongDouble(F80::from(1.0)).is_zero());
}

#[test]
fn a_floating_value_reinterprets_as_an_integer_by_truncation() {
    assert_eq!(ConstValue::Float(3.9).to_i64(), 3);
    assert_eq!(ConstValue::Float(3.9).to_u64(), 3);
    assert_eq!(ConstValue::Double(3.9).to_u64(), 3);
    assert_eq!(ConstValue::LongDouble(F80::from(3.9)).to_u64(), 3);
}

#[test]
fn the_minimum_of_a_signed_type_is_recognised() {
    let fold = ConstFolder;
    assert!(fold.is_min(&ResolvedType::Int, ConstValue::Int(i32::MIN)));
    assert!(fold.is_min(&ResolvedType::Long, ConstValue::Long(-2147483648)));
    assert!(!fold.is_min(&ResolvedType::Int, ConstValue::Int(0)));
    assert!(!fold.is_min(&ResolvedType::UnsignedInt, ConstValue::UnsignedInt(0)));
    assert!(!fold.is_min(&ResolvedType::UnsignedLong, ConstValue::UnsignedLong(0)));
}

#[test]
fn long_double_literals_round_to_the_x87_format() {
    assert_ne!(ConstValue::parse("0.1l").res, ConstValue::LongDouble(F80::from(0.1)));
    assert_eq!(ConstValue::parse("0.1l").res, ConstValue::LongDouble(F80::from("0.1")));
}

#[test]
fn an_exact_long_double_literal_is_the_double_widened() {
    assert_eq!(ConstValue::parse("1.5l").res, ConstValue::LongDouble(F80::from(1.5)));
}
