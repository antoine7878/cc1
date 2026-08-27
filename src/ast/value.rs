use crate::ast_node;
use crate::semantic::{QualifiedType, Sema};
use std::cmp::Ordering;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign,
    Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

// TODO add custom f80
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Int(i32),
    Long(i64),
    UnsignedLong(u64),
    UnsignedInt(u32),
    Float(f32),
    Double(f64),
    LongDouble(f64),
}

ast_node! {
    pub struct ValueNode {
        pub value: Value,
    }
}

impl ValueNode {
    pub fn ty(&self, sema: &Sema) -> QualifiedType {
        let ty = match &self.value {
            Value::Int(_) => sema.builtins.int,
            Value::Long(_) => sema.builtins.long,
            Value::UnsignedInt(_) => sema.builtins.unsigned_int,
            Value::UnsignedLong(_) => sema.builtins.unsigned_long,
            Value::Float(_) => sema.builtins.float,
            Value::Double(_) => sema.builtins.double,
            Value::LongDouble(_) => sema.builtins.long_double,
        };
        QualifiedType::new(ty, false, false)
    }
}

impl Value {
    pub fn get_integer_value(&self) -> Option<u64> {
        match *self {
            Value::Int(c) => Some(c as u64),
            Value::UnsignedInt(c) => Some(c as u64),
            Value::Long(c) => Some(c as u64),
            Value::UnsignedLong(c) => Some(c),
            _ => None,
        }
    }

    fn get_radix(s: &str) -> (&str, u32) {
        if s.starts_with("0x") {
            ("0x", 16)
        } else if s.starts_with("0") {
            ("0", 8)
        } else {
            ("", 10)
        }
    }

    fn get_float_suffix(s: &str) -> &str {
        if s.ends_with('f') {
            "f"
        } else if s.ends_with('l') {
            "l"
        } else {
            ""
        }
    }

    fn get_integer_suffix(s: &str) -> &str {
        if s.ends_with("ul") || s.ends_with("lu") {
            "ul"
        } else if s.ends_with('u') {
            "u"
        } else if s.ends_with('l') {
            "l"
        } else {
            ""
        }
    }

    fn no_integer_prefix(value: u64, radix: u32) -> Self {
        if value <= i32::MAX as u64 {
            Value::Int(value as i32)
        } else if radix != 10 && value <= u32::MAX as u64 {
            Value::UnsignedInt(value as u32)
        } else if value <= i64::MAX as u64 {
            Value::Long(value as i64)
        } else {
            Value::UnsignedLong(value)
        }
    }

    fn parse_float(s: &str) -> Self {
        let s = s.to_lowercase();
        let suffix = Self::get_float_suffix(s.as_str());
        let s = &s[0..(s.len() - suffix.len())];
        match suffix {
            "f" => Value::Float(s.parse::<f32>().unwrap()),
            "l" => Value::LongDouble(s.parse::<f64>().unwrap()),
            _ => Value::Double(s.parse::<f64>().unwrap()),
        }
    }

    fn parse_escape(bytes: &[u8], i: &mut usize) -> u32 {
        let c = bytes[*i];
        *i += 1;
        match c {
            b'a' => 7,
            b'b' => 8,
            b'f' => 12,
            b'n' => 10,
            b'r' => 13,
            b't' => 9,
            b'v' => 11,
            b'x' => {
                let mut value: u32 = 0;
                while *i < bytes.len() && (bytes[*i] as char).is_ascii_hexdigit() {
                    value = value.wrapping_mul(16) + (bytes[*i] as char).to_digit(16).unwrap();
                    *i += 1;
                }
                value
            }
            b'0'..=b'7' => {
                let mut value = (c - b'0') as u32;
                let mut len = 1;
                while *i < bytes.len() && len < 3 && (b'0'..=b'7').contains(&bytes[*i]) {
                    value = value * 8 + (bytes[*i] - b'0') as u32;
                    *i += 1;
                    len += 1;
                }
                value
            }
            _ => c as u32,
        }
    }

    fn parse_char(s: &str) -> Self {
        let prefix = if s.starts_with("L") { "L" } else { "" };
        let s = &s[(prefix.len() + 1)..(s.len() - 1)];
        let bytes = s.as_bytes();
        let mut i = 0;
        let mut value: u32 = 0;
        let mut count = 0;
        while i < bytes.len() {
            let c = if bytes[i] == b'\\' {
                i += 1;
                Self::parse_escape(bytes, &mut i)
            } else {
                i += 1;
                bytes[i - 1] as u32
            };
            value = if prefix.is_empty() { (value << 8) | (c & 0xff) } else { c };
            count += 1;
        }
        if prefix.is_empty() && count == 1 && value & 0x80 != 0 {
            Value::Int((value | 0xffffff00) as i32)
        } else {
            Value::Int(value as i32)
        }
    }

    fn parse_integer(s: &str) -> Self {
        let s = s.to_lowercase();
        let (prefix, radix) = Self::get_radix(s.as_str());
        let suffix = Self::get_integer_suffix(s.as_str());
        let s = &s[prefix.len()..(s.len() - suffix.len())];
        let value = if s.is_empty() { 0 } else { u64::from_str_radix(s, radix).unwrap() };
        match suffix {
            "u" if value > u32::MAX as u64 => Value::UnsignedLong(value),
            "u" => Value::UnsignedInt(value as u32),
            "l" if value > i64::MAX as u64 => Value::UnsignedLong(value),
            "l" => Value::Long(value as i64),
            "ul" => Value::UnsignedLong(value),
            _ => Self::no_integer_prefix(value, radix),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self.usual(*other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::UnsignedInt(a), Value::UnsignedInt(b)) => a == b,
            (Value::Long(a), Value::Long(b)) => a == b,
            (Value::UnsignedLong(a), Value::UnsignedLong(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Double(a), Value::Double(b)) => a == b,
            (Value::LongDouble(a), Value::LongDouble(b)) => a == b,
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.usual(*other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(&b),
            (Value::UnsignedInt(a), Value::UnsignedInt(b)) => a.partial_cmp(&b),
            (Value::Long(a), Value::Long(b)) => a.partial_cmp(&b),
            (Value::UnsignedLong(a), Value::UnsignedLong(b)) => a.partial_cmp(&b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(&b),
            (Value::Double(a), Value::Double(b)) => a.partial_cmp(&b),
            (Value::LongDouble(a), Value::LongDouble(b)) => a.partial_cmp(&b),
            _ => unreachable!(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Int(value as i32)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        let lower = s.to_lowercase();
        if s.contains('\'') {
            Self::parse_char(s)
        } else if !lower.starts_with("0x") && (lower.contains('.') || lower.contains('e')) {
            Self::parse_float(s)
        } else {
            Self::parse_integer(s)
        }
    }
}

/// 6.2.1.1 Characters and integers / 6.2.1.5 Usual arithmetic conversions
/// The ranks are ordered so that the greater of two operand ranks is the common type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    Float,
    Double,
    LongDouble,
}

impl Rank {
    pub fn is_floating(self) -> bool {
        self >= Rank::Float
    }

    pub fn to_integer(self) -> Rank {
        Rank::min(self, Rank::UnsignedLong)
    }
}

impl Value {
    pub fn is_zero(&self) -> bool {
        match *self {
            Value::Int(i) => i == 0,
            Value::UnsignedInt(i) => i == 0,
            Value::Long(i) => i == 0,
            Value::UnsignedLong(i) => i == 0,
            Value::Float(i) => i == 0.,
            Value::Double(i) => i == 0.,
            Value::LongDouble(i) => i == 0.,
        }
    }

    fn rank(self) -> Rank {
        match self {
            Value::Int(_) => Rank::Int,
            Value::UnsignedInt(_) => Rank::UnsignedInt,
            Value::Long(_) => Rank::Long,
            Value::UnsignedLong(_) => Rank::UnsignedLong,
            Value::Float(_) => Rank::Float,
            Value::Double(_) => Rank::Double,
            Value::LongDouble(_) => Rank::LongDouble,
        }
    }

    pub fn to_i64(self) -> i64 {
        match self {
            Value::Int(v) => v as i64,
            Value::UnsignedInt(v) => v as i64,
            Value::Long(v) => v,
            Value::UnsignedLong(v) => v as i64,
            Value::Float(v) => v as i64,
            Value::Double(v) | Value::LongDouble(v) => v as i64,
        }
    }

    pub fn to_u64(self) -> u64 {
        match self {
            Value::Int(v) => v as u64,
            Value::UnsignedInt(v) => v as u64,
            Value::Long(v) => v as u64,
            Value::UnsignedLong(v) => v,
            Value::Float(v) => v as u64,
            Value::Double(v) | Value::LongDouble(v) => v as u64,
        }
    }

    fn to_f64(self) -> f64 {
        match self {
            Value::Int(v) => v as f64,
            Value::UnsignedInt(v) => v as f64,
            Value::Long(v) => v as f64,
            Value::UnsignedLong(v) => v as f64,
            Value::Float(v) => v as f64,
            Value::Double(v) | Value::LongDouble(v) => v,
        }
    }

    pub fn truncate(self, bits: u32, signed: bool) -> Value {
        if bits == 0 || bits >= 64 {
            return if signed { Value::Long(self.to_i64()) } else { Value::UnsignedLong(self.to_u64()) };
        }
        let mask = (1u64 << bits) - 1;
        let raw = if signed { self.to_i64() as u64 } else { self.to_u64() } & mask;
        if signed && raw & (1 << (bits - 1)) != 0 {
            Value::Long((raw | !mask) as i64)
        } else {
            Value::Long(raw as i64)
        }
    }

    pub fn is_floating(self) -> bool {
        self.rank().is_floating()
    }

    pub fn is_true(self) -> bool {
        self != Value::Int(0)
    }

    pub fn logical_not(self) -> Value {
        Value::from(!self.is_true())
    }

    pub fn convert(self, rank: Rank) -> Self {
        match rank {
            Rank::Int => Value::Int(self.to_i64() as i32),
            Rank::UnsignedInt => Value::UnsignedInt(self.to_u64() as u32),
            Rank::Long => Value::Long(self.to_i64()),
            Rank::UnsignedLong => Value::UnsignedLong(self.to_u64()),
            Rank::Float => Value::Float(self.to_f64() as f32),
            Rank::Double => Value::Double(self.to_f64()),
            Rank::LongDouble => Value::LongDouble(self.to_f64()),
        }
    }

    fn usual(self, rhs: Value) -> (Self, Self) {
        let rank = Rank::max(self.rank(), rhs.rank());
        (self.convert(rank), rhs.convert(rank))
    }

    fn usual_integer(self, rhs: Value) -> (Self, Self) {
        let rank = Rank::max(self.rank(), rhs.rank()).to_integer();
        (self.convert(rank), rhs.convert(rank))
    }
}

macro_rules! value_arithmetic {
    ($trait:ident, $method:ident, $wrapping:ident) => {
        impl $trait for Value {
            type Output = Value;

            fn $method(self, rhs: Value) -> Value {
                match self.usual(rhs) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a.$wrapping(b)),
                    (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt(a.$wrapping(b)),
                    (Value::Long(a), Value::Long(b)) => Value::Long(a.$wrapping(b)),
                    (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong(a.$wrapping(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float($trait::$method(a, b)),
                    (Value::Double(a), Value::Double(b)) => Value::Double($trait::$method(a, b)),
                    (Value::LongDouble(a), Value::LongDouble(b)) => Value::LongDouble($trait::$method(a, b)),
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! value_division {
    ($trait:ident, $method:ident, $checked:ident) => {
        impl $trait for Value {
            type Output = Value;

            fn $method(self, rhs: Value) -> Value {
                match self.usual(rhs) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a.$checked(b).unwrap_or(0)),
                    (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt(a.$checked(b).unwrap_or(0)),
                    (Value::Long(a), Value::Long(b)) => Value::Long(a.$checked(b).unwrap_or(0)),
                    (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong(a.$checked(b).unwrap_or(0)),
                    (Value::Float(a), Value::Float(b)) => Value::Float($trait::$method(a, b)),
                    (Value::Double(a), Value::Double(b)) => Value::Double($trait::$method(a, b)),
                    (Value::LongDouble(a), Value::LongDouble(b)) => Value::LongDouble($trait::$method(a, b)),
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! value_bitwise {
    ($trait:ident, $method:ident) => {
        impl $trait for Value {
            type Output = Value;

            fn $method(self, rhs: Value) -> Value {
                match self.usual_integer(rhs) {
                    (Value::Int(a), Value::Int(b)) => Value::Int($trait::$method(a, b)),
                    (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt($trait::$method(a, b)),
                    (Value::Long(a), Value::Long(b)) => Value::Long($trait::$method(a, b)),
                    (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong($trait::$method(a, b)),
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! value_shift {
    ($trait:ident, $method:ident, $wrapping:ident) => {
        impl $trait for Value {
            type Output = Value;

            fn $method(self, rhs: Value) -> Value {
                let count = rhs.convert(rhs.rank().to_integer()).to_u64() as u32;
                match self.convert(self.rank().to_integer()) {
                    Value::Int(a) => Value::Int(a.$wrapping(count)),
                    Value::UnsignedInt(a) => Value::UnsignedInt(a.$wrapping(count)),
                    Value::Long(a) => Value::Long(a.$wrapping(count)),
                    Value::UnsignedLong(a) => Value::UnsignedLong(a.$wrapping(count)),
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! value_assign {
    ($trait:ident, $method:ident, $base:ident, $base_method:ident) => {
        impl $trait for Value {
            fn $method(&mut self, rhs: Value) {
                *self = $base::$base_method(*self, rhs);
            }
        }
    };
}

value_arithmetic!(Add, add, wrapping_add);
value_arithmetic!(Sub, sub, wrapping_sub);
value_arithmetic!(Mul, mul, wrapping_mul);
value_division!(Div, div, checked_div);

impl Rem for Value {
    type Output = Value;

    fn rem(self, rhs: Value) -> Value {
        match self.usual_integer(rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a.checked_rem(b).unwrap_or(0)),
            (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt(a.checked_rem(b).unwrap_or(0)),
            (Value::Long(a), Value::Long(b)) => Value::Long(a.checked_rem(b).unwrap_or(0)),
            (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong(a.checked_rem(b).unwrap_or(0)),
            _ => unreachable!(),
        }
    }
}

value_bitwise!(BitAnd, bitand);
value_bitwise!(BitOr, bitor);
value_bitwise!(BitXor, bitxor);
value_shift!(Shl, shl, wrapping_shl);
value_shift!(Shr, shr, wrapping_shr);

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Value {
        match self {
            Value::Int(v) => Value::Int(v.wrapping_neg()),
            Value::UnsignedInt(v) => Value::UnsignedInt(v.wrapping_neg()),
            Value::Long(v) => Value::Long(v.wrapping_neg()),
            Value::UnsignedLong(v) => Value::UnsignedLong(v.wrapping_neg()),
            Value::Float(v) => Value::Float(-v),
            Value::Double(v) => Value::Double(-v),
            Value::LongDouble(v) => Value::LongDouble(-v),
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Value {
        match self.convert(self.rank().to_integer()) {
            Value::Int(v) => Value::Int(!v),
            Value::UnsignedInt(v) => Value::UnsignedInt(!v),
            Value::Long(v) => Value::Long(!v),
            Value::UnsignedLong(v) => Value::UnsignedLong(!v),
            _ => unreachable!(),
        }
    }
}

value_assign!(AddAssign, add_assign, Add, add);
value_assign!(SubAssign, sub_assign, Sub, sub);
value_assign!(MulAssign, mul_assign, Mul, mul);
value_assign!(DivAssign, div_assign, Div, div);
value_assign!(RemAssign, rem_assign, Rem, rem);
value_assign!(BitAndAssign, bitand_assign, BitAnd, bitand);
value_assign!(BitOrAssign, bitor_assign, BitOr, bitor);
value_assign!(BitXorAssign, bitxor_assign, BitXor, bitxor);
value_assign!(ShlAssign, shl_assign, Shl, shl);
value_assign!(ShrAssign, shr_assign, Shr, shr);
