use std::fmt;
use std::iter::zip;

use crate::arena::ResolveWith;
use crate::ast::Tag;
use crate::context::Context;
use crate::define_interner;
use crate::semantic::{ParamTypes, Sema, TagDefId};
use crate::target::Target;

define_interner!(ResolvedType, ResolvedTypeArena, ResolvedTypeId, Sema, sema, types);

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

    pub fn is_char(&self) -> bool {
        matches!(
            self,
            ResolvedType::Char | ResolvedType::UnsignedChar | ResolvedType::SignedChar
        )
    }

    pub fn is_integral(&self, sema: &Sema) -> bool {
        match self {
            ResolvedType::Tag(id) => (*id).resolve(sema).kind == Tag::Enum,
            _ => self.is_integer(),
        }
    }

    pub fn is_floating(&self) -> bool {
        matches!(
            self,
            ResolvedType::Float | ResolvedType::Double | ResolvedType::LongDouble
        )
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

    pub fn interge_prefix(&self) -> &str {
        match self {
            ResolvedType::Char
            | ResolvedType::SignedChar
            | ResolvedType::Short
            | ResolvedType::Int
            | ResolvedType::Long => "i",
            ResolvedType::UnsignedChar
            | ResolvedType::UnsignedShort
            | ResolvedType::UnsignedInt
            | ResolvedType::UnsignedLong => "u",
            ResolvedType::Float | ResolvedType::Double | ResolvedType::LongDouble => "f",
            _ => "",
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
    pub fn new(types: &mut ResolvedTypeArena, target: &Target) -> Self {
        Self {
            void: types.alloc(ResolvedType::Void),
            char: types.alloc(ResolvedType::Char),
            signed_char: types.alloc(ResolvedType::SignedChar),
            unsigned_char: types.alloc(ResolvedType::UnsignedChar),
            short: types.alloc(ResolvedType::Short),
            unsigned_short: types.alloc(ResolvedType::UnsignedShort),
            int: types.alloc(ResolvedType::Int),
            unsigned_int: types.alloc(ResolvedType::UnsignedInt),
            long: types.alloc(ResolvedType::Long),
            unsigned_long: types.alloc(ResolvedType::UnsignedLong),
            float: types.alloc(ResolvedType::Float),
            double: types.alloc(ResolvedType::Double),
            long_double: types.alloc(ResolvedType::LongDouble),
            ptrdiff_t: types.alloc(target.ptrdiff_t.clone()),
            size_t: types.alloc(target.size_t.clone()),
        }
    }
}

impl ResolvedTypeArena {
    pub fn pointer(&mut self, inner: QualifiedType) -> ResolvedTypeId {
        self.alloc(ResolvedType::Pointer(inner))
    }

    pub fn array(&mut self, elem: QualifiedType, len: Option<usize>) -> ResolvedTypeId {
        self.alloc(ResolvedType::Array { elem, len })
    }

    pub fn function(&mut self, ret: QualifiedType, params: ParamTypes) -> ResolvedTypeId {
        let params = match params {
            ParamTypes::Unspecified => ParamTypes::Unspecified,
            ParamTypes::Prototype { params, is_variadic } => ParamTypes::Prototype {
                params: params.into_iter().map(|param| self.parameter(param)).collect(),
                is_variadic,
            },
        };
        self.alloc(ResolvedType::Function { ret, params })
    }

    pub fn adjust_parameter(&mut self, param: QualifiedType) -> QualifiedType {
        let id = match self.get(param.id).clone() {
            ResolvedType::Array { elem, .. } => self.pointer(elem),
            ResolvedType::Function { .. } => self.pointer(QualifiedType::plain(param.id)),
            _ => return param,
        };
        QualifiedType::new(id, param.is_const, param.is_volatile)
    }

    fn parameter(&mut self, param: QualifiedType) -> QualifiedType {
        let param = self.adjust_parameter(param);
        QualifiedType::plain(param.id)
    }

    pub fn tag(&mut self, id: TagDefId) -> ResolvedTypeId {
        self.alloc(ResolvedType::Tag(id))
    }
}

impl QualifiedType {
    pub fn new(id: ResolvedTypeId, is_const: bool, is_volatile: bool) -> Self {
        QualifiedType {
            id,
            is_const,
            is_volatile,
        }
    }

    pub fn plain(id: ResolvedTypeId) -> Self {
        QualifiedType::new(id, false, false)
    }

    pub fn is_char(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_char()
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
        let l = self.id.resolve(sema).clone();
        let r = other.id.resolve(sema).clone();
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
                    (
                        ParamTypes::Prototype {
                            params: p1,
                            is_variadic,
                        },
                        ParamTypes::Prototype { params: p2, .. },
                    ) => ParamTypes::Prototype {
                        params: zip(&p1, &p2)
                            .map(|(t1, t2)| QualifiedType::composite(t1, sema, t2))
                            .collect::<Option<Vec<_>>>()?,
                        is_variadic,
                    },
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
        match (self.id.resolve(sema), other.id.resolve(sema)) {
            (ResolvedType::Function { ret: r1, params: p1 }, ResolvedType::Function { ret: r2, params: p2 }) => {
                r1.is_compatible(sema, r2) && p1.is_compatible(sema, p2)
            }
            (ResolvedType::Array { elem: e1, len: l1 }, ResolvedType::Array { elem: e2, len: l2 }) => {
                e1.is_compatible(sema, e2) && (l1.is_none() || l2.is_none() || l1 == l2)
            }
            (ResolvedType::Pointer(l), ResolvedType::Pointer(r)) => l.is_compatible(sema, r),
            (ResolvedType::Tag(id), ResolvedType::Int) | (ResolvedType::Int, ResolvedType::Tag(id)) => {
                (*id).resolve(sema).kind == Tag::Enum
            }
            _ => false,
        }
    }

    pub fn describe<'a>(&'a self, sema: &'a Sema, ctx: &'a Context) -> TypeName<'a> {
        TypeName(self, sema, ctx)
    }

    pub fn is_void(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_void()
    }

    pub fn is_complete(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_complete(sema)
    }

    pub fn is_scalar(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_scalar(sema)
    }

    pub fn is_pointer(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_pointer()
    }

    pub fn is_function(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_function()
    }

    pub fn is_object(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_object(sema)
    }

    pub fn is_array(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_array()
    }

    pub fn is_arithmetic(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_arithmetic(sema)
    }

    pub fn is_tag(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_tag()
    }

    pub fn is_integral(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_integral(sema)
    }

    pub fn has_const_member(&self, sema: &Sema) -> bool {
        match self.id.resolve(sema) {
            ResolvedType::Array { elem, .. } => elem.is_const || elem.has_const_member(sema),
            ResolvedType::Tag(id) => sema.tags.get(*id).members.iter().any(|member| {
                member
                    .sym
                    .and_then(|sym| sema.symbols.get(sym).ty)
                    .is_some_and(|ty| ty.is_const || ty.has_const_member(sema))
            }),
            _ => false,
        }
    }

    pub fn is_floating(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_floating()
    }

    pub fn is_integer(&self, sema: &Sema) -> bool {
        self.id.resolve(sema).is_integer()
    }
}

pub struct TypeName<'a>(&'a QualifiedType, &'a Sema, &'a Context);

impl fmt::Display for TypeName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let TypeName(ty, sema, ctx) = *self;
        if ty.is_const {
            f.write_str("const ")?;
        }
        if ty.is_volatile {
            f.write_str("volatile ")?;
        }
        match ty.id.resolve(sema) {
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
                match inner.id.resolve(sema) {
                    ResolvedType::Function { .. } | ResolvedType::Array { .. } => {
                        write!(f, "({})", inner.describe(sema, ctx))?
                    }
                    _ => write!(f, "{}", inner.describe(sema, ctx))?,
                }
                f.write_str(" *")
            }
            &ResolvedType::Tag(id) => {
                let def = id.resolve(sema);
                let name = def.name.map_or("<anonymous>", |n| n.id.resolve(ctx).as_str());
                write!(f, "{} {}", def.kind(), name)
            }
            ResolvedType::Array { elem, len } => {
                let mut base = elem;
                while let ResolvedType::Array { elem: inner, .. } = base.id.resolve(sema) {
                    base = inner;
                }
                write!(f, "{}", base.describe(sema, ctx))?;
                let (mut elem, mut len) = (elem, len);
                loop {
                    f.write_str("[")?;
                    if let Some(len) = len {
                        write!(f, "{len}")?;
                    }
                    f.write_str("]")?;
                    let ResolvedType::Array { elem: inner, len: size } = elem.id.resolve(sema) else { break };
                    (elem, len) = (inner, size);
                }
                Ok(())
            }
            ResolvedType::Function { ret, params } => {
                write!(f, "{}(", ret.describe(sema, ctx))?;
                if let ParamTypes::Prototype { params, is_variadic } = params {
                    if params.is_empty() {
                        f.write_str("void")?;
                    } else {
                        for (i, param) in params.iter().enumerate() {
                            if i > 0 {
                                f.write_str(", ")?;
                            }
                            write!(f, "{}", param.describe(sema, ctx))?;
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
