use crate::{ast::Value, semantic::ResolvedType};

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub size: u32,
    pub align: u32,
}

impl Layout {
    pub const fn new(size: u32, align: u32) -> Self {
        Self { size, align }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub name: &'static str,
    pub char: Layout,
    pub short: Layout,
    pub int: Layout,
    pub long: Layout,
    pub float: Layout,
    pub double: Layout,
    pub long_double: Layout,
    pub pointer: Layout,
    pub size_t: ResolvedType,
    pub ptrdiff_t: ResolvedType,
    pub wchar_t: ResolvedType,
    pub char_signed: bool,
}

pub const I386: Target = Target {
    name: "i386",
    char: Layout::new(1, 1),
    short: Layout::new(2, 2),
    int: Layout::new(4, 4),
    long: Layout::new(4, 4),
    float: Layout::new(4, 4),
    double: Layout::new(8, 4),
    long_double: Layout::new(12, 4),
    pointer: Layout::new(4, 4),
    size_t: ResolvedType::UnsignedInt,
    ptrdiff_t: ResolvedType::Int,
    wchar_t: ResolvedType::Int,
    char_signed: true,
};

pub const X86_64: Target = Target {
    name: "x86_64",
    char: Layout::new(1, 1),
    short: Layout::new(2, 2),
    int: Layout::new(4, 4),
    long: Layout::new(8, 8),
    float: Layout::new(4, 4),
    double: Layout::new(8, 8),
    long_double: Layout::new(16, 16),
    pointer: Layout::new(8, 8),
    size_t: ResolvedType::UnsignedLong,
    ptrdiff_t: ResolvedType::Long,
    wchar_t: ResolvedType::Int,
    char_signed: true,
};

impl Default for Target {
    fn default() -> Self {
        I386
    }
}

impl Target {
    pub fn value_size(&self, value: Value) -> u64 {
        match value {
            Value::Int(_) | Value::UnsignedInt(_) => self.int,
            Value::Long(_) | Value::UnsignedLong(_) => self.long,
            Value::Float(_) => self.float,
            Value::Double(_) => self.double,
            Value::LongDouble(_) => self.long_double,
        }
        .size
        .into()
    }

    pub fn scalar(&self, ty: &ResolvedType) -> Option<Layout> {
        let layout = match ty {
            ResolvedType::Char | ResolvedType::SignedChar | ResolvedType::UnsignedChar => self.char,
            ResolvedType::Short | ResolvedType::UnsignedShort => self.short,
            ResolvedType::Int | ResolvedType::UnsignedInt => self.int,
            ResolvedType::Long | ResolvedType::UnsignedLong => self.long,
            ResolvedType::Float => self.float,
            ResolvedType::Double => self.double,
            ResolvedType::LongDouble => self.long_double,
            ResolvedType::Array { .. } | ResolvedType::Pointer(_) => self.pointer,
            ResolvedType::Void | ResolvedType::Tag(_) => return None,
        };
        Some(layout)
    }

    pub fn cast(&self, ty: &ResolvedType, val: Value) -> Option<Value> {
        if !self.is_integral(ty) {
            return None;
        }
        let v = val.truncate(self.bits(ty)?, self.is_signed(ty));
        Some(match ty {
            ResolvedType::Long => Value::Long(v.to_i64()),
            ResolvedType::UnsignedLong => Value::UnsignedLong(v.to_u64()),
            ResolvedType::UnsignedInt => Value::UnsignedInt(v.to_u64() as u32),
            _ => Value::Int(v.to_i64() as i32),
        })
    }

    pub fn is_integral(&self, ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::Char
                | ResolvedType::SignedChar
                | ResolvedType::UnsignedChar
                | ResolvedType::Short
                | ResolvedType::UnsignedShort
                | ResolvedType::Int
                | ResolvedType::UnsignedInt
                | ResolvedType::Long
                | ResolvedType::UnsignedLong
        )
    }

    pub fn is_signed(&self, ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Char => self.char_signed,
            ResolvedType::SignedChar
            | ResolvedType::Short
            | ResolvedType::Int
            | ResolvedType::Long
            | ResolvedType::Float
            | ResolvedType::Double
            | ResolvedType::LongDouble => true,
            _ => false,
        }
    }

    pub fn bits(&self, ty: &ResolvedType) -> Option<u32> {
        self.scalar(ty).map(|l| l.size * 8)
    }

    pub fn max_value(&self, ty: &ResolvedType) -> Option<u64> {
        if !self.is_integral(ty) {
            return None;
        }
        let bits = self.bits(ty)?;
        let signed = self.is_signed(ty);
        if bits >= 64 {
            return Some(if signed { i64::MAX as u64 } else { u64::MAX });
        }
        Some(if signed { (1u64 << (bits - 1)) - 1 } else { (1u64 << bits) - 1 })
    }

    pub fn min_value(&self, ty: &ResolvedType) -> Option<i64> {
        if !self.is_integral(ty) {
            return None;
        }
        if !self.is_signed(ty) {
            return Some(0);
        }
        let bits = self.bits(ty)?;
        if bits >= 64 {
            return Some(i64::MIN);
        }
        Some(-(1i64 << (bits - 1)))
    }

    pub fn truncate(&self, value: i64, ty: &ResolvedType) -> Option<i64> {
        if !self.is_integral(ty) {
            return None;
        }
        let bits = self.bits(ty)?;
        if bits >= 64 {
            return Some(value);
        }
        let masked = (value as u64) & ((1u64 << bits) - 1);
        Some(if self.is_signed(ty) && masked >> (bits - 1) != 0 {
            (masked | !((1u64 << bits) - 1)) as i64
        } else {
            masked as i64
        })
    }
}
