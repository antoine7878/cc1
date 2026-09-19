use std::cmp::Ordering;
use std::fmt;

use crate::ast::{BinaryOp, F80, UnaryOp, escape};
use crate::ast_node;
use crate::semantic::{Diag, Diagnostic, QualifiedType, ResolvedType, Sema};
use crate::target::Target;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstValue {
    Int(i32),
    Long(i64),
    UnsignedLong(u64),
    UnsignedInt(u32),
    Float(f32),
    Double(f64),
    LongDouble(F80),
}

ast_node! {
    pub struct ConstValueNode {
        pub value: ConstValue,
    }
}

impl ConstValueNode {
    pub fn ty(&self, sema: &Sema) -> QualifiedType {
        let ty = match &self.value {
            ConstValue::Int(_) => sema.builtins.int,
            ConstValue::Long(_) => sema.builtins.long,
            ConstValue::UnsignedInt(_) => sema.builtins.unsigned_int,
            ConstValue::UnsignedLong(_) => sema.builtins.unsigned_long,
            ConstValue::Float(_) => sema.builtins.float,
            ConstValue::Double(_) => sema.builtins.double,
            ConstValue::LongDouble(_) => sema.builtins.long_double,
        };
        QualifiedType::plain(ty)
    }
}

impl fmt::Display for ConstValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstValue::Int(i) => write!(f, "{}", i),
            ConstValue::Long(i) => write!(f, "{}", i),
            ConstValue::UnsignedLong(i) => write!(f, "{}", i),
            ConstValue::UnsignedInt(i) => write!(f, "{}", i),
            ConstValue::Float(i) => write!(f, "{}", i),
            ConstValue::Double(i) => write!(f, "{}", i),
            ConstValue::LongDouble(i) => write!(f, "{}", i),
        }
    }
}

impl ConstValue {
    pub fn get_integer_value(&self) -> Option<u64> {
        match *self {
            ConstValue::Int(c) => Some(c as u64),
            ConstValue::UnsignedInt(c) => Some(c as u64),
            ConstValue::Long(c) => Some(c as u64),
            ConstValue::UnsignedLong(c) => Some(c),
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

    fn integer_candidates(suffix: &str, radix: u32) -> &'static [ResolvedType] {
        use ResolvedType::{Int, Long, UnsignedInt, UnsignedLong};
        match suffix {
            "u" => &[UnsignedInt, UnsignedLong],
            "l" => &[Long, UnsignedLong],
            "ul" => &[UnsignedLong],
            _ if radix == 10 => &[Int, Long, UnsignedLong],
            _ => &[Int, UnsignedInt, Long, UnsignedLong],
        }
    }

    fn parse_float(s: &str) -> Self {
        let s = s.to_lowercase();
        let suffix = Self::get_float_suffix(s.as_str());
        let s = &s[0..(s.len() - suffix.len())];
        match suffix {
            "f" => ConstValue::Float(s.parse::<f32>().unwrap()),
            "l" => ConstValue::LongDouble(F80::from(s)),
            _ => ConstValue::Double(s.parse::<f64>().unwrap()),
        }
    }

    fn parse_char(s: &str, target: &Target) -> Self {
        let prefix = if s.starts_with("L") { "L" } else { "" };
        let s = &s[(prefix.len() + 1)..(s.len() - 1)];
        let narrow = prefix.is_empty();
        let (value, count) = Self::char_sequence(s.as_bytes(), narrow);
        if narrow && count == 1 && target.char_signed && value & 0x80 != 0 {
            ConstValue::Int((value | 0xffffff00) as i32)
        } else {
            ConstValue::Int(value as i32)
        }
    }

    fn char_sequence(bytes: &[u8], narrow: bool) -> (u32, usize) {
        let mut i = 0;
        let mut value: u32 = 0;
        let mut count = 0;
        while i < bytes.len() {
            let c = escape::next(bytes, &mut i);
            value = if narrow { (value << 8) | (c & 0xff) } else { c };
            count += 1;
        }
        (value, count)
    }

    fn parse_integer(s: &str, target: &Target) -> Diag<Self> {
        let s = s.to_lowercase();
        let (prefix, radix) = Self::get_radix(s.as_str());
        let suffix = Self::get_integer_suffix(s.as_str());
        let digits = &s[prefix.len()..(s.len() - suffix.len())];
        let value = Self::digits_value(digits, radix);
        Self::integer_type(value, Self::integer_candidates(suffix, radix), target)
    }

    fn digits_value(digits: &str, radix: u32) -> u64 {
        match digits.is_empty() {
            true => 0,
            false => u64::from_str_radix(digits, radix).unwrap_or(u64::MAX),
        }
    }

    fn integer_type(value: u64, candidates: &[ResolvedType], target: &Target) -> Diag<Self> {
        let fitting = candidates.iter().find(|ty| target.fits(value, ty));
        let diagnostic = fitting.is_none().then_some(Diagnostic::IntegerConstantTooLarge);
        let ty = fitting.or_else(|| candidates.last()).expect("a non empty candidate list");
        let value = target.cast(ty, ConstValue::UnsignedLong(value)).expect("an integer type");
        Diag::new(value, diagnostic)
    }

    pub fn parse(s: &str, target: &Target) -> Diag<Self> {
        let lower = s.to_lowercase();
        if s.contains('\'') {
            Diag::ok(Self::parse_char(s, target))
        } else if !lower.starts_with("0x") && (lower.contains('.') || lower.contains('e')) {
            Diag::ok(Self::parse_float(s))
        } else {
            Self::parse_integer(s, target)
        }
    }
}

impl From<bool> for ConstValue {
    fn from(value: bool) -> Self {
        ConstValue::Int(value as i32)
    }
}

impl ConstValue {
    pub fn is_negative(&self) -> bool {
        match *self {
            ConstValue::Int(i) => i < 0,
            ConstValue::UnsignedInt(_) => false,
            ConstValue::Long(i) => i < 0,
            ConstValue::UnsignedLong(_) => false,
            ConstValue::Float(i) => i < 0.,
            ConstValue::Double(i) => i < 0.,
            ConstValue::LongDouble(i) => i.is_negative(),
        }
    }

    pub fn is_greater_or_eq(&self, v: u32) -> bool {
        match *self {
            ConstValue::Int(i) => i >= v as i32,
            ConstValue::UnsignedInt(i) => i >= v,
            ConstValue::Long(i) => i >= v as i64,
            ConstValue::UnsignedLong(i) => i >= v as u64,
            ConstValue::Float(i) => i >= v as f32,
            ConstValue::Double(i) => i >= v as f64,
            ConstValue::LongDouble(i) => i >= F80::from(u64::from(v)),
        }
    }

    pub fn is_zero(&self) -> bool {
        match *self {
            ConstValue::Int(i) => i == 0,
            ConstValue::UnsignedInt(i) => i == 0,
            ConstValue::Long(i) => i == 0,
            ConstValue::UnsignedLong(i) => i == 0,
            ConstValue::Float(i) => i == 0.,
            ConstValue::Double(i) => i == 0.,
            ConstValue::LongDouble(i) => i.is_zero(),
        }
    }

    pub fn to_i64(self) -> i64 {
        match self {
            ConstValue::Int(v) => v as i64,
            ConstValue::UnsignedInt(v) => v as i64,
            ConstValue::Long(v) => v,
            ConstValue::UnsignedLong(v) => v as i64,
            ConstValue::Float(v) => v as i64,
            ConstValue::Double(v) => v as i64,
            ConstValue::LongDouble(v) => i64::from(v),
        }
    }

    pub fn to_u64(self) -> u64 {
        match self {
            ConstValue::Int(v) => v as u64,
            ConstValue::UnsignedInt(v) => v as u64,
            ConstValue::Long(v) => v as u64,
            ConstValue::UnsignedLong(v) => v,
            ConstValue::Float(v) => v as u64,
            ConstValue::Double(v) => v as u64,
            ConstValue::LongDouble(v) => u64::from(v),
        }
    }

    fn to_f64(self) -> f64 {
        match self {
            ConstValue::Int(v) => v as f64,
            ConstValue::UnsignedInt(v) => v as f64,
            ConstValue::Long(v) => v as f64,
            ConstValue::UnsignedLong(v) => v as f64,
            ConstValue::Float(v) => v as f64,
            ConstValue::Double(v) => v,
            ConstValue::LongDouble(v) => f64::from(v),
        }
    }

    fn to_f80(self) -> F80 {
        match self {
            ConstValue::Int(v) => F80::from(i64::from(v)),
            ConstValue::UnsignedInt(v) => F80::from(u64::from(v)),
            ConstValue::Long(v) => F80::from(v),
            ConstValue::UnsignedLong(v) => F80::from(v),
            ConstValue::Float(v) => F80::from(f64::from(v)),
            ConstValue::Double(v) => F80::from(v),
            ConstValue::LongDouble(v) => v,
        }
    }

    pub fn truncate(self, bits: u32, signed: bool) -> ConstValue {
        if bits == 0 || bits >= 64 {
            return if signed { ConstValue::Long(self.to_i64()) } else { ConstValue::UnsignedLong(self.to_u64()) };
        }
        let mask = (1u64 << bits) - 1;
        let raw = if signed { self.to_i64() as u64 } else { self.to_u64() } & mask;
        if signed && raw & (1 << (bits - 1)) != 0 {
            ConstValue::Long((raw | !mask) as i64)
        } else {
            ConstValue::Long(raw as i64)
        }
    }

    pub fn is_floating(self) -> bool {
        matches!(self, ConstValue::Float(_) | ConstValue::Double(_) | ConstValue::LongDouble(_))
    }

    pub fn is_true(self) -> bool {
        !self.is_zero()
    }

    pub fn logical_not(self) -> ConstValue {
        ConstValue::from(!self.is_true())
    }
}

pub struct ConstFolder<'a> {
    target: &'a Target,
}

impl<'a> ConstFolder<'a> {
    pub fn new(target: &'a Target) -> Self {
        Self { target }
    }

    pub fn convert(&self, ty: &ResolvedType, value: ConstValue) -> Option<ConstValue> {
        match ty {
            ResolvedType::Float => Some(ConstValue::Float(value.to_f64() as f32)),
            ResolvedType::Double => Some(ConstValue::Double(value.to_f64())),
            ResolvedType::LongDouble => Some(ConstValue::LongDouble(value.to_f80())),
            _ => self.target.cast(ty, value),
        }
    }

    fn shift(&self, ty: &ResolvedType, op: BinaryOp, lhs: ConstValue, rhs: ConstValue) -> ConstValue {
        let count = rhs.to_i64() as u32;
        let shifted = match (op, self.convert(ty, lhs).expect("an integer type")) {
            (BinaryOp::Left, ConstValue::Int(a)) => ConstValue::Int(a.wrapping_shl(count)),
            (BinaryOp::Left, ConstValue::UnsignedInt(a)) => ConstValue::UnsignedInt(a.wrapping_shl(count)),
            (BinaryOp::Left, ConstValue::Long(a)) => ConstValue::Long(a.wrapping_shl(count)),
            (BinaryOp::Left, ConstValue::UnsignedLong(a)) => ConstValue::UnsignedLong(a.wrapping_shl(count)),
            (BinaryOp::Right, ConstValue::Int(a)) => ConstValue::Int(a.wrapping_shr(count)),
            (BinaryOp::Right, ConstValue::UnsignedInt(a)) => ConstValue::UnsignedInt(a.wrapping_shr(count)),
            (BinaryOp::Right, ConstValue::Long(a)) => ConstValue::Long(a.wrapping_shr(count)),
            (BinaryOp::Right, ConstValue::UnsignedLong(a)) => ConstValue::UnsignedLong(a.wrapping_shr(count)),
            _ => unreachable!(),
        };
        self.convert(ty, shifted).expect("an integer type")
    }

    pub fn binary(&self, ty: &ResolvedType, op: BinaryOp, lhs: ConstValue, rhs: ConstValue) -> Diag<ConstValue> {
        if matches!(op, BinaryOp::Left | BinaryOp::Right) {
            return Diag::ok(self.shift(ty, op, lhs, rhs));
        }
        if ty.is_floating() {
            return Diag::ok(self.floating(ty, op, lhs, rhs));
        }
        if !self.target.is_signed(ty) {
            return Diag::ok(self.unsigned(ty, op, lhs, rhs));
        }
        self.signed(ty, op, lhs, rhs)
    }

    fn floating(&self, ty: &ResolvedType, op: BinaryOp, lhs: ConstValue, rhs: ConstValue) -> ConstValue {
        use BinaryOp::{Add, Div, Mul, Sub};

        if matches!(ty, ResolvedType::LongDouble) && self.target.long_double.size == 16 {
            let (a, b) = (lhs.to_f80(), rhs.to_f80());
            return ConstValue::LongDouble(match op {
                Add => a + b,
                Sub => a - b,
                Mul => a * b,
                Div => a / b,
                _ => unreachable!(),
            });
        }
        let (a, b) = (lhs.to_f64(), rhs.to_f64());
        let r = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            _ => unreachable!(),
        };
        self.convert(ty, ConstValue::Double(r)).expect("a floating type")
    }

    fn unsigned(&self, ty: &ResolvedType, op: BinaryOp, lhs: ConstValue, rhs: ConstValue) -> ConstValue {
        use BinaryOp::{Add, BitAnd, BitOr, BitXor, Div, Mod, Mul, Sub};

        let (a, b) = (u128::from(lhs.to_u64()), u128::from(rhs.to_u64()));
        let r = match op {
            Add => a.wrapping_add(b),
            Sub => a.wrapping_sub(b),
            Mul => a.wrapping_mul(b),
            Div => a.checked_div(b).unwrap_or(0),
            Mod => a.checked_rem(b).unwrap_or(0),
            BitAnd => a & b,
            BitOr => a | b,
            BitXor => a ^ b,
            _ => unreachable!(),
        };
        self.convert(ty, ConstValue::UnsignedLong(r as u64)).expect("an integer type")
    }

    fn signed(&self, ty: &ResolvedType, op: BinaryOp, lhs: ConstValue, rhs: ConstValue) -> Diag<ConstValue> {
        use BinaryOp::{Add, BitAnd, BitOr, BitXor, Div, Mod, Mul, Sub};

        let (a, b) = (i128::from(lhs.to_i64()), i128::from(rhs.to_i64()));
        let r = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a.checked_div(b).unwrap_or(0),
            Mod => a.checked_rem(b).unwrap_or(0),
            BitAnd => a & b,
            BitOr => a | b,
            BitXor => a ^ b,
            _ => unreachable!(),
        };
        let value = self.convert(ty, ConstValue::Long(r as i64)).expect("an integer type");
        Diag::new(value, self.overflow(ty, op, r))
    }

    fn overflow(&self, ty: &ResolvedType, op: BinaryOp, r: i128) -> Option<Diagnostic> {
        let min = i128::from(self.target.min_value(ty).unwrap_or(i64::MIN));
        let max = i128::from(self.target.max_value(ty).unwrap_or(i64::MAX as u64) as i64);
        let out_of_range = matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) && (r < min || r > max);
        out_of_range.then_some(Diagnostic::ArithmeticOverflow)
    }

    pub fn unary(&self, ty: &ResolvedType, op: UnaryOp, value: ConstValue) -> Diag<ConstValue> {
        match op {
            UnaryOp::Minus => self.neg(ty, value),
            UnaryOp::BitNot => Diag::ok(self.bit_not(ty, value)),
            _ => unreachable!(),
        }
    }

    fn neg(&self, ty: &ResolvedType, value: ConstValue) -> Diag<ConstValue> {
        if matches!(ty, ResolvedType::LongDouble) {
            return Diag::ok(ConstValue::LongDouble(-value.to_f80()));
        }
        if ty.is_floating() {
            let negated = self.convert(ty, ConstValue::Double(-value.to_f64()));
            return Diag::ok(negated.expect("a floating type"));
        }
        self.binary(ty, BinaryOp::Sub, ConstValue::Int(0), value)
    }

    fn bit_not(&self, ty: &ResolvedType, value: ConstValue) -> ConstValue {
        let complement = match self.convert(ty, value).expect("an integer type") {
            ConstValue::Int(v) => ConstValue::Int(!v),
            ConstValue::UnsignedInt(v) => ConstValue::UnsignedInt(!v),
            ConstValue::Long(v) => ConstValue::Long(!v),
            ConstValue::UnsignedLong(v) => ConstValue::UnsignedLong(!v),
            _ => unreachable!(),
        };
        self.convert(ty, complement).expect("an integer type")
    }

    pub fn compare(&self, lhs: ConstValue, rhs: ConstValue) -> Option<Ordering> {
        match (lhs, rhs) {
            (ConstValue::Int(a), ConstValue::Int(b)) => a.partial_cmp(&b),
            (ConstValue::UnsignedInt(a), ConstValue::UnsignedInt(b)) => a.partial_cmp(&b),
            (ConstValue::Long(a), ConstValue::Long(b)) => a.partial_cmp(&b),
            (ConstValue::UnsignedLong(a), ConstValue::UnsignedLong(b)) => a.partial_cmp(&b),
            (ConstValue::Float(a), ConstValue::Float(b)) => a.partial_cmp(&b),
            (ConstValue::Double(a), ConstValue::Double(b)) => a.partial_cmp(&b),
            (ConstValue::LongDouble(a), ConstValue::LongDouble(b)) => a.partial_cmp(&b),
            _ => None,
        }
    }

    pub fn is_min(&self, ty: &ResolvedType, value: ConstValue) -> bool {
        self.target.is_signed(ty) && self.target.min_value(ty).is_some_and(|min| value.to_i64() == min)
    }
}
