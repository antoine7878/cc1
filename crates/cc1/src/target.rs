use crate::{
    ast::{ConstValue, F80},
    semantic::ResolvedType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatFormat {
    Ieee64,
    X87,
}

impl FloatFormat {
    pub const fn llvm(self) -> &'static str {
        match self {
            FloatFormat::Ieee64 => "double",
            FloatFormat::X87 => "x86_fp80",
        }
    }

    pub fn round(self, value: F80) -> F80 {
        match self {
            FloatFormat::X87 => value,
            FloatFormat::Ieee64 => F80::from(f64::from(value)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub size: u32,
    pub align: u32,
}

impl Layout {
    pub const fn new(size: u32, align: u32) -> Self {
        Self { size, align }
    }

    pub fn llvm(&self) -> &str {
        match self {
            Layout { size: 4, .. } => "float",
            Layout { size: 8, .. } => "double",
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Target {
    pub name: &'static str,
    pub char: Layout,
    pub short: Layout,
    pub int: Layout,
    pub long: Layout,
    pub float: Layout,
    pub double: Layout,
    pub long_double: Layout,
    pub long_double_format: FloatFormat,
    pub pointer: Layout,
    pub size_t: ResolvedType,
    pub ptrdiff_t: ResolvedType,
    pub wchar_t: ResolvedType,
    pub char_signed: bool,
    pub byte_size: u32,
    pub triple: &'static str,
    pub datalayout: &'static str,
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
    long_double_format: FloatFormat::X87,
    pointer: Layout::new(4, 4),
    size_t: ResolvedType::UnsignedInt,
    ptrdiff_t: ResolvedType::Int,
    wchar_t: ResolvedType::Long,
    char_signed: true,
    byte_size: 8,
    triple: "i386-pc-linux-gnu",
    datalayout: "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-f64:32:64-f80:32-n8:16:32-S128",
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
    long_double_format: FloatFormat::X87,
    pointer: Layout::new(8, 8),
    size_t: ResolvedType::UnsignedLong,
    ptrdiff_t: ResolvedType::Long,
    wchar_t: ResolvedType::Int,
    char_signed: true,
    byte_size: 8,
    triple: "x86_64-pc-linux-gnu",
    datalayout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
};

pub const ARM64_DARWIN: Target = Target {
    name: "arm64",
    char: Layout::new(1, 1),
    short: Layout::new(2, 2),
    int: Layout::new(4, 4),
    long: Layout::new(8, 8),
    float: Layout::new(4, 4),
    double: Layout::new(8, 8),
    long_double: Layout::new(8, 8),
    long_double_format: FloatFormat::Ieee64,
    pointer: Layout::new(8, 8),
    size_t: ResolvedType::UnsignedLong,
    ptrdiff_t: ResolvedType::Long,
    wchar_t: ResolvedType::Int,
    char_signed: true,
    byte_size: 8,
    triple: "arm64-apple-macosx26.0.0",
    datalayout: "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32",
};

impl Default for Target {
    fn default() -> Self {
        X86_64
    }
}

impl Target {
    pub fn value_size(&self, value: ConstValue) -> u64 {
        match value {
            ConstValue::Int(_) | ConstValue::UnsignedInt(_) => self.int,
            ConstValue::Long(_) | ConstValue::UnsignedLong(_) => self.long,
            ConstValue::Float(_) => self.float,
            ConstValue::Double(_) => self.double,
            ConstValue::LongDouble(_) => self.long_double,
        }
        .size
        .into()
    }

    pub fn layout(&self, ty: &ResolvedType) -> Option<Layout> {
        let layout = match ty {
            ResolvedType::Char | ResolvedType::SignedChar | ResolvedType::UnsignedChar => self.char,
            ResolvedType::Short | ResolvedType::UnsignedShort => self.short,
            ResolvedType::Int | ResolvedType::UnsignedInt => self.int,
            ResolvedType::Long | ResolvedType::UnsignedLong => self.long,
            ResolvedType::Float => self.float,
            ResolvedType::Double => self.double,
            ResolvedType::LongDouble => self.long_double,
            ResolvedType::Function { .. } | ResolvedType::Pointer(_) => self.pointer,
            ResolvedType::Array { .. } | ResolvedType::Void | ResolvedType::Tag(_) => return None,
        };
        Some(layout)
    }

    pub fn cast(&self, ty: &ResolvedType, val: ConstValue) -> Option<ConstValue> {
        if !self.is_integral(ty) {
            return None;
        }
        let v = val.truncate(self.bits(ty)?, self.is_signed(ty));
        Some(match ty {
            ResolvedType::Long => ConstValue::Long(v.to_i64()),
            ResolvedType::UnsignedLong => ConstValue::UnsignedLong(v.to_u64()),
            ResolvedType::UnsignedInt => ConstValue::UnsignedInt(v.to_u64() as u32),
            _ => ConstValue::Int(v.to_i64() as i32),
        })
    }

    pub fn fits(&self, value: u64, ty: &ResolvedType) -> bool {
        self.max_value(ty).is_some_and(|max| value <= max)
    }

    pub fn is_integral(&self, ty: &ResolvedType) -> bool {
        ty.is_integer()
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
        self.layout(ty).map(|l| l.size * 8)
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
