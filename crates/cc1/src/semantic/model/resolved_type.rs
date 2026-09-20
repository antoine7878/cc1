use std::fmt::{self};
use std::iter::zip;

use crate::ast::{ConstValue, Tag};
use crate::define_interner;
use crate::semantic::layout::{self, Layout};
use crate::semantic::{ParamTypes, Sema, TagDefId};

define_interner!(ResolvedType, ResolvedTypeInterner, ResolvedTypeId);

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum ResolvedType {
    Void,
    Char,
    SignedChar,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    Float,
    Double,
    LongDouble,
    Array { elem: QualifiedType, len: Option<usize> },
    Function { ret: QualifiedType, params: ParamTypes },
    Pointer(QualifiedType),
    Tag(TagDefId),
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub struct QualifiedType {
    pub id: ResolvedTypeId,
    pub is_const: bool,
    pub is_volatile: bool,
}

impl ResolvedType {
    pub fn pointee(&self) -> Option<QualifiedType> {
        match self {
            ResolvedType::Pointer(i) => Some(*i),
            _ => None,
        }
    }
    pub fn is_complete(&self, sema: &Sema) -> bool {
        match self {
            ResolvedType::Void => false,
            ResolvedType::Array { len, .. } => len.is_some(),
            ResolvedType::Tag(id) => sema.tags.get(*id).is_complete,
            _ => true,
        }
    }

    pub fn is_void(&self) -> bool {
        self == &ResolvedType::Void
    }

    pub fn is_scalar(&self, sema: &Sema) -> bool {
        self.is_arithmetic(sema) || self.is_pointer()
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, ResolvedType::Pointer(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, ResolvedType::Function { .. })
    }

    pub fn is_object(&self, sema: &Sema) -> bool {
        self.is_complete(sema) && !self.is_function()
    }

    pub fn is_array(&self) -> bool {
        matches!(self, ResolvedType::Array { .. })
    }

    pub fn is_arithmetic(&self, sema: &Sema) -> bool {
        self.is_integral(sema) || self.is_floating()
    }

    pub fn is_tag(&self) -> bool {
        matches!(self, ResolvedType::Tag(_))
    }

    pub fn is_record(&self, sema: &Sema) -> bool {
        match self {
            ResolvedType::Tag(id) => (*id).resolve_with(sema).kind != Tag::Enum,
            _ => false,
        }
    }

    pub fn is_char(&self) -> bool {
        matches!(self, ResolvedType::Char | ResolvedType::UnsignedChar | ResolvedType::SignedChar)
    }

    pub fn is_integral(&self, sema: &Sema) -> bool {
        match self {
            ResolvedType::Tag(id) => (*id).resolve_with(sema).kind == Tag::Enum,
            _ => self.is_integer(),
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
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

    pub fn is_floating(&self) -> bool {
        matches!(self, ResolvedType::Float | ResolvedType::Double | ResolvedType::LongDouble)
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            ResolvedType::Char
                | ResolvedType::SignedChar
                | ResolvedType::Short
                | ResolvedType::Int
                | ResolvedType::Long
        )
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            ResolvedType::UnsignedChar
                | ResolvedType::UnsignedShort
                | ResolvedType::UnsignedInt
                | ResolvedType::UnsignedLong
        )
    }

    pub fn layout(&self) -> Option<Layout> {
        let layout = match self {
            ResolvedType::Char | ResolvedType::SignedChar | ResolvedType::UnsignedChar => layout::CHAR,
            ResolvedType::Short | ResolvedType::UnsignedShort => layout::SHORT,
            ResolvedType::Int | ResolvedType::UnsignedInt => layout::INT,
            ResolvedType::Long | ResolvedType::UnsignedLong => layout::LONG,
            ResolvedType::Float => layout::FLOAT,
            ResolvedType::Double => layout::DOUBLE,
            ResolvedType::LongDouble => layout::LONG_DOUBLE,
            ResolvedType::Function { .. } | ResolvedType::Pointer(_) => layout::POINTER,
            ResolvedType::Array { .. } | ResolvedType::Void | ResolvedType::Tag(_) => return None,
        };
        Some(layout)
    }

    pub fn bits(&self) -> Option<u32> {
        self.layout().map(|l| l.size * layout::CHAR_BIT)
    }

    pub fn max_value(&self) -> Option<u64> {
        if !self.is_integer() {
            return None;
        }
        let bits = self.bits()?;
        Some(if self.is_signed() { (1u64 << (bits - 1)) - 1 } else { (1u64 << bits) - 1 })
    }

    pub fn min_value(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        if !self.is_signed() {
            return Some(0);
        }
        let bits = self.bits()?;
        Some(-(1i64 << (bits - 1)))
    }

    pub fn cast(&self, val: ConstValue) -> Option<ConstValue> {
        if !self.is_integer() {
            return None;
        }
        let v = val.truncate(self.bits()?, self.is_signed());
        Some(match self {
            ResolvedType::Long => ConstValue::Long(v.to_i64()),
            ResolvedType::UnsignedLong => ConstValue::UnsignedLong(v.to_u64()),
            ResolvedType::UnsignedInt => ConstValue::UnsignedInt(v.to_u64() as u32),
            _ => ConstValue::Int(v.to_i64() as i32),
        })
    }

    pub fn fits(&self, value: u64) -> bool {
        self.max_value().is_some_and(|max| value <= max)
    }

    pub fn class(&self) -> Option<NumericClass> {
        if self.is_floating() {
            Some(NumericClass::Float)
        } else if self.is_signed() {
            Some(NumericClass::Signed)
        } else if self.is_unsigned() {
            Some(NumericClass::Unsigned)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Builtins {
    pub void: ResolvedTypeId,
    pub char: ResolvedTypeId,
    pub signed_char: ResolvedTypeId,
    pub unsigned_char: ResolvedTypeId,
    pub short: ResolvedTypeId,
    pub unsigned_short: ResolvedTypeId,
    pub int: ResolvedTypeId,
    pub unsigned_int: ResolvedTypeId,
    pub long: ResolvedTypeId,
    pub unsigned_long: ResolvedTypeId,
    pub float: ResolvedTypeId,
    pub double: ResolvedTypeId,
    pub long_double: ResolvedTypeId,
    pub ptrdiff_t: ResolvedTypeId,
    pub size_t: ResolvedTypeId,
}

impl Builtins {
    pub fn new(types: &mut ResolvedTypeInterner) -> Self {
        let int = types.intern(ResolvedType::Int);
        let unsigned_int = types.intern(ResolvedType::UnsignedInt);
        Self {
            void: types.intern(ResolvedType::Void),
            char: types.intern(ResolvedType::Char),
            signed_char: types.intern(ResolvedType::SignedChar),
            unsigned_char: types.intern(ResolvedType::UnsignedChar),
            short: types.intern(ResolvedType::Short),
            unsigned_short: types.intern(ResolvedType::UnsignedShort),
            int,
            unsigned_int,
            long: types.intern(ResolvedType::Long),
            unsigned_long: types.intern(ResolvedType::UnsignedLong),
            float: types.intern(ResolvedType::Float),
            double: types.intern(ResolvedType::Double),
            long_double: types.intern(ResolvedType::LongDouble),
            ptrdiff_t: int,
            size_t: unsigned_int,
        }
    }
}

impl ResolvedTypeInterner {
    pub fn pointer(&mut self, inner: QualifiedType) -> ResolvedTypeId {
        self.intern(ResolvedType::Pointer(inner))
    }

    pub fn array(&mut self, elem: QualifiedType, len: Option<usize>) -> ResolvedTypeId {
        self.intern(ResolvedType::Array { elem, len })
    }

    pub fn function(&mut self, ret: QualifiedType, params: ParamTypes) -> ResolvedTypeId {
        let params = match params {
            ParamTypes::Unspecified => ParamTypes::Unspecified,
            ParamTypes::Prototype { params, is_variadic } => ParamTypes::Prototype {
                params: params.into_iter().map(|param| self.param(param)).collect(),
                is_variadic,
            },
        };
        self.intern(ResolvedType::Function { ret, params })
    }

    pub fn adjust_param(&mut self, param: QualifiedType) -> QualifiedType {
        let id = match self.get(param.id).clone() {
            ResolvedType::Array { elem, .. } => self.pointer(elem),
            ResolvedType::Function { .. } => self.pointer(QualifiedType::plain(param.id)),
            _ => return param,
        };
        QualifiedType::new(id, param.is_const, param.is_volatile)
    }

    fn param(&mut self, param: QualifiedType) -> QualifiedType {
        let param = self.adjust_param(param);
        QualifiedType::plain(param.id)
    }

    pub fn tag(&mut self, id: TagDefId) -> ResolvedTypeId {
        self.intern(ResolvedType::Tag(id))
    }
}

impl QualifiedType {
    pub fn new(id: ResolvedTypeId, is_const: bool, is_volatile: bool) -> Self {
        QualifiedType { id, is_const, is_volatile }
    }

    pub fn plain(id: ResolvedTypeId) -> Self {
        QualifiedType::new(id, false, false)
    }

    pub fn is_char(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_char()
    }

    pub fn same_qualifiers_as(&self, other: &Self) -> bool {
        self.is_const == other.is_const && self.is_volatile == other.is_volatile
    }

    pub fn has_qualifiers_of(&self, other: &Self) -> bool {
        self.is_const >= other.is_const && self.is_volatile >= other.is_volatile
    }

    pub fn composite(&self, sema: &mut Sema, other: &Self) -> Option<Self> {
        if !self.is_compatible(sema, other) {
            return None;
        }
        if self == other {
            return Some(*self);
        }
        let l = self.id.resolve_with(sema).clone();
        let r = other.id.resolve_with(sema).clone();
        match (l, r) {
            (ResolvedType::Array { elem: e1, len: l1 }, ResolvedType::Array { elem: e2, len: l2 }) => {
                let elem = Self::composite(&e1, sema, &e2)?;
                let ty = sema.types.array(elem, l1.or(l2));
                Some(QualifiedType::new(ty, self.is_const, self.is_volatile))
            }
            (ResolvedType::Pointer(i1), ResolvedType::Pointer(i2)) => {
                let inner = Self::composite(&i1, sema, &i2)?;
                let ty = sema.types.pointer(inner);
                Some(QualifiedType::new(ty, self.is_const, self.is_volatile))
            }
            (ResolvedType::Function { params: p1, ret: r1 }, ResolvedType::Function { params: p2, ret: r2 }) => {
                let ret = Self::composite(&r1, sema, &r2)?;
                let params = match (p1, p2) {
                    (ParamTypes::Unspecified, ParamTypes::Unspecified) => ParamTypes::Unspecified,
                    (p @ ParamTypes::Prototype { .. }, ParamTypes::Unspecified)
                    | (ParamTypes::Unspecified, p @ ParamTypes::Prototype { .. }) => p,
                    (ParamTypes::Prototype { params: p1, is_variadic }, ParamTypes::Prototype { params: p2, .. }) => {
                        ParamTypes::Prototype {
                            params: zip(&p1, &p2)
                                .map(|(t1, t2)| QualifiedType::composite(t1, sema, t2))
                                .collect::<Option<Vec<_>>>()?,
                            is_variadic,
                        }
                    }
                };
                let ty = sema.types.function(ret, params);
                Some(QualifiedType::new(ty, self.is_const, self.is_volatile))
            }
            _ => Some(*self),
        }
    }

    pub fn unqualified(&self) -> Self {
        QualifiedType::plain(self.id)
    }

    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        self.same_qualifiers_as(other) && self.is_compatible_ignoring_qualifiers(sema, other)
    }

    pub fn is_compatible_ignoring_qualifiers(&self, sema: &Sema, other: &Self) -> bool {
        if self.id == other.id {
            return true;
        }
        match (self.id.resolve_with(sema), other.id.resolve_with(sema)) {
            (ResolvedType::Function { ret: r1, params: p1 }, ResolvedType::Function { ret: r2, params: p2 }) => {
                r1.is_compatible(sema, r2) && p1.is_compatible(sema, p2)
            }
            (ResolvedType::Array { elem: e1, len: l1 }, ResolvedType::Array { elem: e2, len: l2 }) => {
                e1.is_compatible(sema, e2) && (l1.is_none() || l2.is_none() || l1 == l2)
            }
            (ResolvedType::Pointer(l), ResolvedType::Pointer(r)) => l.is_compatible(sema, r),
            (ResolvedType::Tag(id), ResolvedType::Int) | (ResolvedType::Int, ResolvedType::Tag(id)) => {
                (*id).resolve_with(sema).kind == Tag::Enum
            }
            _ => false,
        }
    }

    pub fn is_void(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_void()
    }

    pub fn is_complete(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_complete(sema)
    }

    pub fn is_scalar(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_scalar(sema)
    }

    pub fn is_pointer(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_pointer()
    }

    pub fn is_function(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_function()
    }

    pub fn is_object(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_object(sema)
    }

    pub fn is_array(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_array()
    }

    pub fn is_arithmetic(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_arithmetic(sema)
    }

    pub fn is_tag(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_tag()
    }

    pub fn is_record(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_record(sema)
    }

    pub fn is_integral(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_integral(sema)
    }

    pub fn has_const_member(&self, sema: &Sema) -> bool {
        match self.id.resolve_with(sema) {
            ResolvedType::Array { elem, .. } => elem.is_const || elem.has_const_member(sema),
            ResolvedType::Tag(id) => sema.tags.get(*id).members.iter().any(|member| {
                member
                    .symbol
                    .map(|sym| sema.symbols.get(sym).ty)
                    .is_some_and(|ty| ty.is_const || ty.has_const_member(sema))
            }),
            _ => false,
        }
    }

    pub fn is_integer(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_integer()
    }

    pub fn is_floating(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_floating()
    }

    pub fn is_signed(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_signed()
    }

    pub fn is_unsigned(&self, sema: &Sema) -> bool {
        self.id.resolve_with(sema).is_unsigned()
    }

    pub fn class(&self, sema: &Sema) -> Option<NumericClass> {
        self.id.resolve_with(sema).class()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumericClass {
    Signed,
    Unsigned,
    Float,
}

pub struct TypeDisplay<'a> {
    ty: &'a QualifiedType,
    sema: &'a Sema,
}

impl QualifiedType {
    pub fn display<'a>(&'a self, sema: &'a Sema) -> TypeDisplay<'a> {
        TypeDisplay { ty: self, sema }
    }
}

impl fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = self.ty;
        if ty.is_const {
            f.write_str("const ")?;
        }
        if ty.is_volatile {
            f.write_str("volatile ")?;
        }
        match ty.id.resolve_with(self.sema) {
            ResolvedType::Void => f.write_str("void"),
            ResolvedType::Char => f.write_str("char"),
            ResolvedType::SignedChar => f.write_str("signed char"),
            ResolvedType::UnsignedChar => f.write_str("unsigned char"),
            ResolvedType::Short => f.write_str("short"),
            ResolvedType::UnsignedShort => f.write_str("unsigned short"),
            ResolvedType::Int => f.write_str("int"),
            ResolvedType::UnsignedInt => f.write_str("unsigned int"),
            ResolvedType::Long => f.write_str("long"),
            ResolvedType::UnsignedLong => f.write_str("unsigned long"),
            ResolvedType::Float => f.write_str("float"),
            ResolvedType::Double => f.write_str("double"),
            ResolvedType::LongDouble => f.write_str("long double"),
            ResolvedType::Pointer(inner) => {
                match inner.id.resolve_with(self.sema) {
                    ResolvedType::Function { .. } | ResolvedType::Array { .. } => {
                        write!(f, "({})", inner.display(self.sema))?
                    }
                    _ => write!(f, "{}", inner.display(self.sema))?,
                }
                f.write_str(" *")
            }
            &ResolvedType::Tag(id) => {
                let def = id.resolve_with(self.sema);
                let name = def.name.map_or("<anonymous>", |n| n.id.resolve().as_str());
                write!(f, "{} {}", def.kind.symbol_kind(), name)
            }
            ResolvedType::Array { elem, len } => {
                let mut base = elem;
                while let ResolvedType::Array { elem: inner, .. } = base.id.resolve_with(self.sema) {
                    base = inner;
                }
                write!(f, "{}", base.display(self.sema))?;
                let (mut elem, mut len) = (elem, len);
                loop {
                    f.write_str("[")?;
                    if let Some(len) = len {
                        write!(f, "{len}")?;
                    }
                    f.write_str("]")?;
                    let ResolvedType::Array { elem: inner, len: size } = elem.id.resolve_with(self.sema) else {
                        break;
                    };
                    (elem, len) = (inner, size);
                }
                Ok(())
            }
            ResolvedType::Function { ret, params } => {
                write!(f, "{}(", ret.display(self.sema))?;
                if let ParamTypes::Prototype { params, is_variadic } = params {
                    if params.is_empty() {
                        f.write_str("void")?;
                    } else {
                        for (i, param) in params.iter().enumerate() {
                            if i > 0 {
                                f.write_str(", ")?;
                            }
                            write!(f, "{}", param.display(self.sema))?;
                        }
                    }
                    if *is_variadic {
                        f.write_str(", ...")?;
                    }
                }
                f.write_str(")")
            }
        }
    }
}
