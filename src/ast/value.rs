use crate::ast_node;
use crate::semantic::{Diagnosis, QualifiedType, ResolvedType, Sema};
use crate::target::Target;
use std::cmp::Ordering;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Sub};

// TODO add custom f80
/// `PartialEq` compares the representation, which is what an AST node needs; comparing two
/// constants the way C does is `Fold::eq`, since that needs the usual arithmetic conversions.
#[derive(Clone, Copy, Debug, PartialEq)]
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

    /// 6.1.3.2 Integer constants
    /// The type of an integer constant is the first of the corresponding list in which its value
    /// can be represented.
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

    fn parse_char(s: &str, target: &Target) -> Self {
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
        if prefix.is_empty() && count == 1 && target.char_signed && value & 0x80 != 0 {
            Value::Int((value | 0xffffff00) as i32)
        } else {
            Value::Int(value as i32)
        }
    }

    fn parse_integer(s: &str, target: &Target) -> (Self, Option<Diagnosis>) {
        let s = s.to_lowercase();
        let (prefix, radix) = Self::get_radix(s.as_str());
        let suffix = Self::get_integer_suffix(s.as_str());
        let digits = &s[prefix.len()..(s.len() - suffix.len())];
        let value = match digits.is_empty() {
            true => 0,
            false => u64::from_str_radix(digits, radix).unwrap_or(u64::MAX),
        };
        let candidates = Self::integer_candidates(suffix, radix);
        let fitting = candidates.iter().find(|ty| target.fits(value, ty));
        let diagnosis = fitting.is_none().then_some(Diagnosis::IntegerConstantTooLarge);
        let ty = fitting
            .or_else(|| candidates.last())
            .expect("a non empty candidate list");
        let value = target.cast(ty, Value::UnsignedLong(value)).expect("an integer type");
        (value, diagnosis)
    }

    /// 6.1.3 Constants
    pub fn parse(s: &str, target: &Target) -> (Self, Option<Diagnosis>) {
        let lower = s.to_lowercase();
        if s.contains('\'') {
            (Self::parse_char(s, target), None)
        } else if !lower.starts_with("0x") && (lower.contains('.') || lower.contains('e')) {
            (Self::parse_float(s), None)
        } else {
            Self::parse_integer(s, target)
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Int(value as i32)
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

    fn resolved(self) -> ResolvedType {
        match self {
            Rank::Int => ResolvedType::Int,
            Rank::UnsignedInt => ResolvedType::UnsignedInt,
            Rank::Long => ResolvedType::Long,
            Rank::UnsignedLong => ResolvedType::UnsignedLong,
            Rank::Float => ResolvedType::Float,
            Rank::Double => ResolvedType::Double,
            Rank::LongDouble => ResolvedType::LongDouble,
        }
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
        !self.is_zero()
    }

    pub fn logical_not(self) -> Value {
        Value::from(!self.is_true())
    }
}

/// 6.2.1.5 Usual arithmetic conversions
/// Folding needs the target: both the common type of two operands and the width a result is
/// truncated to depend on the size of the integer types.
pub struct Fold<'a> {
    target: &'a Target,
}

macro_rules! fold_arithmetic {
    ($method:ident, $trait:ident, $wrapping:ident) => {
        pub fn $method(&self, lhs: Value, rhs: Value) -> Value {
            let value = match self.usual(lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Value::Int(a.$wrapping(b)),
                (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt(a.$wrapping(b)),
                (Value::Long(a), Value::Long(b)) => Value::Long(a.$wrapping(b)),
                (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong(a.$wrapping(b)),
                (Value::Float(a), Value::Float(b)) => Value::Float($trait::$method(a, b)),
                (Value::Double(a), Value::Double(b)) => Value::Double($trait::$method(a, b)),
                (Value::LongDouble(a), Value::LongDouble(b)) => Value::LongDouble($trait::$method(a, b)),
                _ => unreachable!(),
            };
            self.narrow(value)
        }
    };
}

macro_rules! fold_division {
    ($method:ident, $trait:ident, $checked:ident) => {
        pub fn $method(&self, lhs: Value, rhs: Value) -> Value {
            let value = match self.usual(lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Value::Int(a.$checked(b).unwrap_or(0)),
                (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt(a.$checked(b).unwrap_or(0)),
                (Value::Long(a), Value::Long(b)) => Value::Long(a.$checked(b).unwrap_or(0)),
                (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong(a.$checked(b).unwrap_or(0)),
                (Value::Float(a), Value::Float(b)) => Value::Float($trait::$method(a, b)),
                (Value::Double(a), Value::Double(b)) => Value::Double($trait::$method(a, b)),
                (Value::LongDouble(a), Value::LongDouble(b)) => Value::LongDouble($trait::$method(a, b)),
                _ => unreachable!(),
            };
            self.narrow(value)
        }
    };
}

macro_rules! fold_bitwise {
    ($method:ident, $trait:ident) => {
        pub fn $method(&self, lhs: Value, rhs: Value) -> Value {
            let value = match self.usual_integer(lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Value::Int($trait::$method(a, b)),
                (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt($trait::$method(a, b)),
                (Value::Long(a), Value::Long(b)) => Value::Long($trait::$method(a, b)),
                (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong($trait::$method(a, b)),
                _ => unreachable!(),
            };
            self.narrow(value)
        }
    };
}

macro_rules! fold_shift {
    ($method:ident, $wrapping:ident) => {
        pub fn $method(&self, lhs: Value, rhs: Value) -> Value {
            let count = self.convert(rhs, rhs.rank().to_integer()).to_u64() as u32;
            let value = match self.convert(lhs, lhs.rank().to_integer()) {
                Value::Int(a) => Value::Int(a.$wrapping(count)),
                Value::UnsignedInt(a) => Value::UnsignedInt(a.$wrapping(count)),
                Value::Long(a) => Value::Long(a.$wrapping(count)),
                Value::UnsignedLong(a) => Value::UnsignedLong(a.$wrapping(count)),
                _ => unreachable!(),
            };
            self.narrow(value)
        }
    };
}

macro_rules! fold_relational {
    ($method:ident, $($ordering:path)|+) => {
        pub fn $method(&self, lhs: Value, rhs: Value) -> bool {
            matches!(self.cmp(lhs, rhs), $(Some($ordering))|+)
        }
    };
}

impl<'a> Fold<'a> {
    pub fn new(target: &'a Target) -> Self {
        Self { target }
    }

    /// 6.2.1.5 The greater rank of the two operands is the common type, except that a long int
    /// which cannot represent every value of an unsigned int meets it at unsigned long int.
    fn common(&self, lhs: Value, rhs: Value) -> Rank {
        let (l, r) = (lhs.rank(), rhs.rank());
        let rank = Rank::max(l, r);
        if rank == Rank::Long && Rank::min(l, r) == Rank::UnsignedInt && self.target.long.size <= self.target.int.size {
            return Rank::UnsignedLong;
        }
        rank
    }

    fn usual(&self, lhs: Value, rhs: Value) -> (Value, Value) {
        let rank = self.common(lhs, rhs);
        (self.convert(lhs, rank), self.convert(rhs, rank))
    }

    fn usual_integer(&self, lhs: Value, rhs: Value) -> (Value, Value) {
        let rank = self.common(lhs, rhs).to_integer();
        (self.convert(lhs, rank), self.convert(rhs, rank))
    }

    /// A result is representable in the type of the operands it was computed from.
    fn narrow(&self, value: Value) -> Value {
        self.convert(value, value.rank())
    }

    /// 6.2.1.2 When a value of integral type is converted to another integral type, the value is
    /// truncated to the width the target gives that type.
    pub fn convert(&self, value: Value, rank: Rank) -> Value {
        match rank {
            Rank::Float => Value::Float(value.to_f64() as f32),
            Rank::Double => Value::Double(value.to_f64()),
            Rank::LongDouble => Value::LongDouble(value.to_f64()),
            _ => self.target.cast(&rank.resolved(), value).expect("an integer type"),
        }
    }

    pub fn cmp(&self, lhs: Value, rhs: Value) -> Option<Ordering> {
        match self.usual(lhs, rhs) {
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

    /// 6.3.5 The result of INT_MIN / -1 is not representable in the type of the operands.
    pub fn is_min(&self, value: Value) -> bool {
        let ty = value.rank().resolved();
        self.target.is_signed(&ty)
            && self
                .target
                .min_value(&ty)
                .is_some_and(|min| self.eq(value, Value::Long(min)))
    }

    pub fn neg(&self, value: Value) -> Value {
        let value = match value {
            Value::Int(v) => Value::Int(v.wrapping_neg()),
            Value::UnsignedInt(v) => Value::UnsignedInt(v.wrapping_neg()),
            Value::Long(v) => Value::Long(v.wrapping_neg()),
            Value::UnsignedLong(v) => Value::UnsignedLong(v.wrapping_neg()),
            Value::Float(v) => Value::Float(-v),
            Value::Double(v) => Value::Double(-v),
            Value::LongDouble(v) => Value::LongDouble(-v),
        };
        self.narrow(value)
    }

    pub fn bit_not(&self, value: Value) -> Value {
        let value = match self.convert(value, value.rank().to_integer()) {
            Value::Int(v) => Value::Int(!v),
            Value::UnsignedInt(v) => Value::UnsignedInt(!v),
            Value::Long(v) => Value::Long(!v),
            Value::UnsignedLong(v) => Value::UnsignedLong(!v),
            _ => unreachable!(),
        };
        self.narrow(value)
    }

    /// 6.3.5 The operands of the % operator shall have integral type.
    pub fn rem(&self, lhs: Value, rhs: Value) -> Value {
        let value = match self.usual_integer(lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a.checked_rem(b).unwrap_or(0)),
            (Value::UnsignedInt(a), Value::UnsignedInt(b)) => Value::UnsignedInt(a.checked_rem(b).unwrap_or(0)),
            (Value::Long(a), Value::Long(b)) => Value::Long(a.checked_rem(b).unwrap_or(0)),
            (Value::UnsignedLong(a), Value::UnsignedLong(b)) => Value::UnsignedLong(a.checked_rem(b).unwrap_or(0)),
            _ => unreachable!(),
        };
        self.narrow(value)
    }

    fold_arithmetic!(add, Add, wrapping_add);
    fold_arithmetic!(sub, Sub, wrapping_sub);
    fold_arithmetic!(mul, Mul, wrapping_mul);
    fold_division!(div, Div, checked_div);
    fold_bitwise!(bitand, BitAnd);
    fold_bitwise!(bitor, BitOr);
    fold_bitwise!(bitxor, BitXor);
    fold_shift!(shl, wrapping_shl);
    fold_shift!(shr, wrapping_shr);

    fold_relational!(eq, Ordering::Equal);
    fold_relational!(ne, Ordering::Less | Ordering::Greater);
    fold_relational!(lt, Ordering::Less);
    fold_relational!(gt, Ordering::Greater);
    fold_relational!(le, Ordering::Less | Ordering::Equal);
    fold_relational!(ge, Ordering::Greater | Ordering::Equal);
}
