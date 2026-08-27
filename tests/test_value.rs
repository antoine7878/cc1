use cc1::ast::{Rank, Value};

fn repr(value: Value) -> String {
    format!("{value:?}")
}

macro_rules! literal {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(repr(Value::from($src)), $expected, "Value::from({:?})", $src);
        }
    };
}

macro_rules! fold {
    ($name:ident, $expr:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(repr($expr), $expected, "{}", stringify!($expr));
        }
    };
}

literal!(literal_zero, "0", "Int(0)");
literal!(literal_decimal, "42", "Int(42)");
literal!(literal_int_max, "2147483647", "Int(2147483647)");
literal!(literal_decimal_above_int_max, "2147483648", "Long(2147483648)");
literal!(literal_octal, "010", "Int(8)");
literal!(literal_octal_max, "017777777777", "Int(2147483647)");
literal!(literal_hexadecimal, "0x10", "Int(16)");
literal!(literal_hexadecimal_upper, "0XFF", "Int(255)");
literal!(
    literal_hexadecimal_above_int_max,
    "0x80000000",
    "UnsignedInt(2147483648)"
);
literal!(literal_octal_above_int_max, "020000000000", "UnsignedInt(2147483648)");
literal!(literal_above_unsigned_int_max, "0x100000000", "Long(4294967296)");
literal!(
    literal_above_long_max,
    "0xffffffffffffffff",
    "UnsignedLong(18446744073709551615)"
);

literal!(literal_unsigned_suffix, "1u", "UnsignedInt(1)");
literal!(literal_unsigned_suffix_upper, "1U", "UnsignedInt(1)");
literal!(
    literal_unsigned_suffix_promotes,
    "4294967296u",
    "UnsignedLong(4294967296)"
);
literal!(literal_long_suffix, "1l", "Long(1)");
literal!(literal_long_suffix_upper, "1L", "Long(1)");
literal!(literal_unsigned_long_suffix, "1ul", "UnsignedLong(1)");
literal!(literal_long_unsigned_suffix, "1lu", "UnsignedLong(1)");
literal!(literal_unsigned_long_suffix_upper, "1UL", "UnsignedLong(1)");

literal!(literal_double, "1.5", "Double(1.5)");
literal!(literal_double_leading_dot, ".5", "Double(0.5)");
literal!(literal_double_exponent, "1e3", "Double(1000.0)");
literal!(literal_double_negative_exponent, "1e-3", "Double(0.001)");
literal!(literal_float_suffix, "1.5f", "Float(1.5)");
literal!(literal_float_suffix_upper, "1.5F", "Float(1.5)");
literal!(literal_long_double_suffix, "1.5l", "LongDouble(1.5)");

literal!(literal_char, "'a'", "Int(97)");
literal!(literal_char_digit, "'0'", "Int(48)");
literal!(literal_char_escape_newline, "'\\n'", "Int(10)");
literal!(literal_char_escape_tab, "'\\t'", "Int(9)");
literal!(literal_char_escape_null, "'\\0'", "Int(0)");
literal!(literal_char_escape_backslash, "'\\\\'", "Int(92)");
literal!(literal_char_escape_quote, "'\\''", "Int(39)");
literal!(literal_char_escape_octal, "'\\101'", "Int(65)");
literal!(literal_char_escape_hexadecimal, "'\\x41'", "Int(65)");
literal!(literal_char_is_sign_extended, "'\\377'", "Int(-1)");
literal!(literal_char_hexadecimal_is_sign_extended, "'\\xff'", "Int(-1)");
literal!(literal_char_multi_is_packed, "'ab'", "Int(24930)");
literal!(literal_wide_char, "L'a'", "Int(97)");
literal!(literal_wide_char_is_not_sign_extended, "L'\\xff'", "Int(255)");

fold!(add_int, Value::Int(1) + Value::Int(2), "Int(3)");
fold!(add_wraps, Value::Int(i32::MAX) + Value::Int(1), "Int(-2147483648)");
fold!(
    sub_unsigned_wraps,
    Value::UnsignedInt(1) - Value::UnsignedInt(2),
    "UnsignedInt(4294967295)"
);
fold!(mul_wraps, Value::Int(65536) * Value::Int(65536), "Int(0)");
fold!(div_int, Value::Int(7) / Value::Int(2), "Int(3)");
fold!(
    div_negative_truncates_toward_zero,
    Value::Int(-7) / Value::Int(2),
    "Int(-3)"
);
fold!(div_by_zero_is_zero, Value::Int(7) / Value::Int(0), "Int(0)");
fold!(div_overflow_is_zero, Value::Int(i32::MIN) / Value::Int(-1), "Int(0)");
fold!(rem_int, Value::Int(7) % Value::Int(2), "Int(1)");
fold!(rem_keeps_sign_of_dividend, Value::Int(-7) % Value::Int(2), "Int(-1)");
fold!(rem_by_zero_is_zero, Value::Int(7) % Value::Int(0), "Int(0)");

fold!(
    add_promotes_to_unsigned,
    Value::Int(1) + Value::UnsignedInt(0),
    "UnsignedInt(1)"
);
fold!(add_promotes_to_long, Value::Int(1) + Value::Long(1), "Long(2)");
fold!(
    add_promotes_to_double,
    Value::Int(1) + Value::Double(0.5),
    "Double(1.5)"
);
fold!(
    div_promotes_to_double,
    Value::Int(1) / Value::Double(2.0),
    "Double(0.5)"
);
fold!(
    add_promotes_to_long_double,
    Value::Double(1.0) + Value::LongDouble(1.0),
    "LongDouble(2.0)"
);

fold!(bitand_int, Value::Int(6) & Value::Int(3), "Int(2)");
fold!(bitor_int, Value::Int(6) | Value::Int(3), "Int(7)");
fold!(bitxor_int, Value::Int(6) ^ Value::Int(3), "Int(5)");
fold!(
    bitand_promotes_to_unsigned,
    Value::Int(-1) & Value::UnsignedInt(255),
    "UnsignedInt(255)"
);
fold!(shift_left, Value::Int(1) << Value::Int(4), "Int(16)");
fold!(
    shift_left_into_sign_bit,
    Value::Int(1) << Value::Int(31),
    "Int(-2147483648)"
);
fold!(
    shift_left_out_of_range_wraps_count,
    Value::Int(1) << Value::Int(32),
    "Int(1)"
);
fold!(shift_right_is_arithmetic, Value::Int(-8) >> Value::Int(1), "Int(-4)");
fold!(
    shift_right_unsigned_is_logical,
    Value::UnsignedInt(2147483648) >> Value::Int(31),
    "UnsignedInt(1)"
);
fold!(
    shift_keeps_left_operand_type,
    Value::Long(1) << Value::Int(1),
    "Long(2)"
);

fold!(neg_int, -Value::Int(1), "Int(-1)");
fold!(neg_wraps, -Value::Int(i32::MIN), "Int(-2147483648)");
fold!(neg_double, -Value::Double(1.5), "Double(-1.5)");
fold!(bitnot_int, !Value::Int(0), "Int(-1)");
fold!(bitnot_unsigned, !Value::UnsignedInt(0), "UnsignedInt(4294967295)");

fold!(
    truncate_signed_sign_extends,
    Value::Int(200).truncate(8, true),
    "Long(-56)"
);
fold!(
    truncate_unsigned_keeps_value,
    Value::Int(200).truncate(8, false),
    "Long(200)"
);
fold!(
    truncate_unsigned_masks,
    Value::Int(-1).truncate(16, false),
    "Long(65535)"
);
fold!(truncate_full_width_signed, Value::Int(5).truncate(64, true), "Long(5)");
fold!(
    truncate_full_width_unsigned,
    Value::Int(-1).truncate(64, false),
    "UnsignedLong(18446744073709551615)"
);
fold!(
    truncate_zero_width_is_full_width,
    Value::Int(-1).truncate(0, false),
    "UnsignedLong(18446744073709551615)"
);

fold!(convert_to_int, Value::Long(300).convert(Rank::Int), "Int(300)");
fold!(
    convert_to_int_wraps,
    Value::Long(4294967297).convert(Rank::Int),
    "Int(1)"
);
fold!(
    convert_to_unsigned_long,
    Value::Int(-1).convert(Rank::UnsignedLong),
    "UnsignedLong(18446744073709551615)"
);
fold!(convert_to_double, Value::Int(3).convert(Rank::Double), "Double(3.0)");
fold!(
    convert_from_double_truncates,
    Value::Double(3.9).convert(Rank::Int),
    "Int(3)"
);

fold!(logical_not_of_zero, Value::Int(0).logical_not(), "Int(1)");
fold!(logical_not_of_nonzero, Value::Int(42).logical_not(), "Int(0)");
fold!(logical_not_of_zero_double, Value::Double(0.0).logical_not(), "Int(1)");
fold!(from_true, Value::from(true), "Int(1)");
fold!(from_false, Value::from(false), "Int(0)");

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
    assert!(Value::Int(1) == Value::Long(1));
    assert!(Value::Int(1) == Value::Double(1.0));
    assert!(Value::Int(-1) == Value::UnsignedInt(u32::MAX));
    assert!(Value::Int(1) != Value::Int(2));
}

#[test]
fn ordering_applies_usual_conversions() {
    assert!(Value::Int(1) < Value::Int(2));
    assert!(Value::Int(1) < Value::Double(1.5));
    assert!(Value::Int(-1) > Value::UnsignedInt(0));
    assert!(Value::Long(-1) < Value::Long(0));
}

#[test]
fn compound_assignment_matches_binary_operator() {
    let mut value = Value::Int(1);
    value += Value::Int(2);
    assert_eq!(repr(value), "Int(3)");
    value *= Value::Int(4);
    assert_eq!(repr(value), "Int(12)");
    value -= Value::Int(2);
    assert_eq!(repr(value), "Int(10)");
    value /= Value::Int(3);
    assert_eq!(repr(value), "Int(3)");
    value %= Value::Int(2);
    assert_eq!(repr(value), "Int(1)");
    value <<= Value::Int(3);
    assert_eq!(repr(value), "Int(8)");
    value >>= Value::Int(1);
    assert_eq!(repr(value), "Int(4)");
    value |= Value::Int(1);
    assert_eq!(repr(value), "Int(5)");
    value &= Value::Int(3);
    assert_eq!(repr(value), "Int(1)");
    value ^= Value::Int(3);
    assert_eq!(repr(value), "Int(2)");
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
