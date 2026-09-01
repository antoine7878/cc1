use cc1::ast::{Fold, Rank, Value};
use cc1::semantic::{Diag, Diagnosis};
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

/// `repr`'s common `Option<Value> -> String` formatting, wrapped for callers (like
/// `fold!`/`fold_overflow!` below) that hand in either a bare `Value` or a `Diag<Value>`.
fn repr(value: impl FoldValue) -> String {
    crate::common::repr(Some(value.fold_value()))
}

/// Folding is done for a target: on i386 int and long are both 32 bits.
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

/// A signed arithmetic result outside its type wraps and is reported (6.3).
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

fold!(add_int, add(Value::Int(1), Value::Int(2)), "Int(3)");
fold_overflow!(add_wraps, add(Value::Int(i32::MAX), Value::Int(1)), "Int(-2147483648)");
fold!(
    sub_unsigned_wraps,
    sub(Value::UnsignedInt(1), Value::UnsignedInt(2)),
    "UnsignedInt(4294967295)"
);
fold_overflow!(mul_wraps, mul(Value::Int(65536), Value::Int(65536)), "Int(0)");
fold!(div_int, div(Value::Int(7), Value::Int(2)), "Int(3)");
fold!(
    div_negative_truncates_toward_zero,
    div(Value::Int(-7), Value::Int(2)),
    "Int(-3)"
);
fold!(rem_int, rem(Value::Int(7), Value::Int(2)), "Int(1)");
fold!(
    rem_keeps_sign_of_dividend,
    rem(Value::Int(-7), Value::Int(2)),
    "Int(-1)"
);

fold!(
    add_promotes_to_unsigned,
    add(Value::Int(1), Value::UnsignedInt(0)),
    "UnsignedInt(1)"
);
fold!(add_promotes_to_long, add(Value::Int(1), Value::Long(1)), "Long(2)");
// 6.2.1.5 a long int that cannot represent every unsigned int meets it at unsigned long int, which
// is the case on i386 where both are 32 bits.
fold!(
    unsigned_int_and_long_meet_at_unsigned_long_on_i386,
    add(Value::UnsignedInt(1), Value::Long(-1)),
    "UnsignedLong(0)"
);
fold!(
    add_promotes_to_double,
    add(Value::Int(1), Value::Double(0.5)),
    "Double(1.5)"
);
fold!(
    div_promotes_to_double,
    div(Value::Int(1), Value::Double(2.0)),
    "Double(0.5)"
);
fold!(
    add_promotes_to_long_double,
    add(Value::Double(1.0), Value::LongDouble(1.0)),
    "LongDouble(2.0)"
);

fold!(bitand_int, bitand(Value::Int(6), Value::Int(3)), "Int(2)");
fold!(bitor_int, bitor(Value::Int(6), Value::Int(3)), "Int(7)");
fold!(bitxor_int, bitxor(Value::Int(6), Value::Int(3)), "Int(5)");
fold!(
    bitand_promotes_to_unsigned,
    bitand(Value::Int(-1), Value::UnsignedInt(255)),
    "UnsignedInt(255)"
);
fold!(shift_left, shl(Value::Int(1), Value::Int(4)), "Int(16)");
fold!(shift_left_into_sign_bit, shl(Value::Int(1), Value::Int(31)), "Int(-2147483648)");
#[test]
fn a_shift_count_at_or_past_the_operand_width_is_out_of_range() {
    let fold = Fold::new(&I386);
    assert!(fold.shift_out_of_range(Value::Int(1), Value::Int(32)));
    assert!(fold.shift_out_of_range(Value::Int(1), Value::Int(-1)));
    assert!(!fold.shift_out_of_range(Value::Int(1), Value::Int(31)));
}
fold!(shift_right_is_arithmetic, shr(Value::Int(-8), Value::Int(1)), "Int(-4)");
fold!(
    shift_right_unsigned_is_logical,
    shr(Value::UnsignedInt(2147483648), Value::Int(31)),
    "UnsignedInt(1)"
);
fold!(
    shift_keeps_left_operand_type,
    shl(Value::Long(1), Value::Int(1)),
    "Long(2)"
);

fold!(neg_int, neg(Value::Int(1)), "Int(-1)");
fold_overflow!(neg_wraps, neg(Value::Int(i32::MIN)), "Int(-2147483648)");
fold!(neg_double, neg(Value::Double(1.5)), "Double(-1.5)");
fold!(bitnot_int, bit_not(Value::Int(0)), "Int(-1)");
fold!(
    bitnot_unsigned,
    bit_not(Value::UnsignedInt(0)),
    "UnsignedInt(4294967295)"
);

// 6.2.1.2 an integral conversion truncates to the width the target gives the destination type.
fold!(convert_to_int, convert(Value::Long(300), Rank::Int), "Int(300)");
fold!(
    convert_to_int_wraps,
    convert(Value::Long(4294967297), Rank::Int),
    "Int(1)"
);
fold!(
    convert_to_unsigned_long_is_32_bits_on_i386,
    convert(Value::Int(-1), Rank::UnsignedLong),
    "UnsignedLong(4294967295)"
);
fold!(
    convert_to_long_is_32_bits_on_i386,
    convert(Value::UnsignedLong(4294967296), Rank::Long),
    "Long(0)"
);
fold!(convert_to_double, convert(Value::Int(3), Rank::Double), "Double(3.0)");
fold!(
    convert_from_double_truncates,
    convert(Value::Double(3.9), Rank::Int),
    "Int(3)"
);

#[test]
fn a_wider_target_keeps_the_whole_value() {
    let fold = Fold::new(&X86_64);
    assert_eq!(
        repr(fold.convert(Value::UnsignedLong(4294967296), Rank::Long)),
        "Long(4294967296)"
    );
    assert_eq!(
        repr(fold.convert(Value::Int(-1), Rank::UnsignedLong)),
        "UnsignedLong(18446744073709551615)"
    );
}

// 6.2.1.5 on a target whose long int represents every unsigned int the pair meets at long int.
#[test]
fn unsigned_int_and_long_meet_at_long_on_x86_64() {
    let fold = Fold::new(&X86_64);
    assert_eq!(repr(fold.add(Value::UnsignedInt(1), Value::Long(-1))), "Long(0)");
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
fn equality_applies_usual_conversions() {
    let fold = Fold::new(&I386);
    assert!(fold.eq(Value::Int(1), Value::Long(1)));
    assert!(fold.eq(Value::Int(1), Value::Double(1.0)));
    assert!(fold.eq(Value::Int(-1), Value::UnsignedInt(u32::MAX)));
    assert!(fold.ne(Value::Int(1), Value::Int(2)));
}

#[test]
fn ordering_applies_usual_conversions() {
    let fold = Fold::new(&I386);
    assert!(fold.lt(Value::Int(1), Value::Int(2)));
    assert!(fold.lt(Value::Int(1), Value::Double(1.5)));
    assert!(fold.gt(Value::Int(-1), Value::UnsignedInt(0)));
    assert!(fold.lt(Value::Long(-1), Value::Long(0)));
    assert!(fold.le(Value::Int(2), Value::Int(2)));
    assert!(fold.ge(Value::Int(2), Value::Int(2)));
}

#[test]
fn every_operator_folds_in_sequence() {
    let fold = Fold::new(&I386);
    let mut value = Value::Int(1);
    value = fold.add(value, Value::Int(2)).res;
    assert_eq!(repr(value), "Int(3)");
    value = fold.mul(value, Value::Int(4)).res;
    assert_eq!(repr(value), "Int(12)");
    value = fold.sub(value, Value::Int(2)).res;
    assert_eq!(repr(value), "Int(10)");
    value = fold.div(value, Value::Int(3));
    assert_eq!(repr(value), "Int(3)");
    value = fold.rem(value, Value::Int(2));
    assert_eq!(repr(value), "Int(1)");
    value = fold.shl(value, Value::Int(3));
    assert_eq!(repr(value), "Int(8)");
    value = fold.shr(value, Value::Int(1));
    assert_eq!(repr(value), "Int(4)");
    value = fold.bitor(value, Value::Int(1));
    assert_eq!(repr(value), "Int(5)");
    value = fold.bitand(value, Value::Int(3));
    assert_eq!(repr(value), "Int(1)");
    value = fold.bitxor(value, Value::Int(3));
    assert_eq!(repr(value), "Int(2)");
}

// 6.3.5 INT_MIN / -1 is not representable in the type of the operands.
#[test]
fn the_minimum_of_a_signed_type_is_recognised() {
    let fold = Fold::new(&I386);
    assert!(fold.is_min(Value::Int(i32::MIN)));
    assert!(fold.is_min(Value::Long(-2147483648)));
    assert!(!fold.is_min(Value::Int(0)));
    assert!(!fold.is_min(Value::UnsignedInt(0)));
    assert!(!fold.is_min(Value::UnsignedLong(0)));
}

#[test]
fn rank_is_ordered_from_int_to_long_double() {
    assert!(Rank::Int < Rank::UnsignedInt);
    assert!(Rank::UnsignedInt < Rank::Long);
    assert!(Rank::Long < Rank::UnsignedLong);
    assert!(Rank::UnsignedLong < Rank::Float);
    assert!(Rank::Float < Rank::Double);
    assert!(Rank::Double < Rank::LongDouble);
}

#[test]
fn rank_is_floating_covers_the_real_ranks() {
    assert!(Rank::Float.is_floating());
    assert!(Rank::Double.is_floating());
    assert!(Rank::LongDouble.is_floating());
    assert!(!Rank::Int.is_floating());
    assert!(!Rank::UnsignedLong.is_floating());
}

#[test]
fn rank_to_integer_caps_at_unsigned_long() {
    assert_eq!(Rank::Int.to_integer(), Rank::Int);
    assert_eq!(Rank::UnsignedLong.to_integer(), Rank::UnsignedLong);
    assert_eq!(Rank::Float.to_integer(), Rank::UnsignedLong);
    assert_eq!(Rank::LongDouble.to_integer(), Rank::UnsignedLong);
}
