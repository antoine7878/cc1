use crate::arena::ResolveWith;
use crate::ast::Tag;
use crate::context::Context;
use crate::define_interner;
use crate::semantic::{ParamTypes, Sema, TagDefId};
use crate::target::Target;

define_interner!(
    ResolvedType,
    ResolvedTypeArena,
    ResolvedTypeId,
    crate::semantic::Sema,
    types
);

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
    // 6.1.2.5 An array type of unknown size is an incomplete type. A structure or union type of
    // unknown content is an incomplete type.
    pub fn is_complete(&self, sema: &Sema) -> bool {
        match self {
            ResolvedType::Void => false,
            ResolvedType::Array { len, .. } => len.is_some(),
            ResolvedType::Tag(id) => sema.tags.get(*id).is_complete,
            _ => true,
        }
    }

    // 6.1.2.5 Arithmetic types and pointer types are collectively called scalar types.
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

    // 6.1.2.5 Integral and floating types are collectively called arithmetic types.
    pub fn is_arithmetic(&self, sema: &Sema) -> bool {
        self.is_integral(sema) || self.is_floating()
    }

    // 6.1.2.5 The type char, the signed and unsigned integer types, and the enumerated types
    // are collectively called integral types.
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
            ResolvedType::Function { .. } => self.pointer(QualifiedType::new(param.id, false, false)),
            _ => return param,
        };
        QualifiedType::new(id, param.is_const, param.is_volatile)
    }

    fn parameter(&mut self, param: QualifiedType) -> QualifiedType {
        let param = self.adjust_parameter(param);
        QualifiedType::new(param.id, false, false)
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

    pub fn same_qualifiers_as(&self, other: &Self) -> bool {
        self.is_const == other.is_const && self.is_volatile == other.is_volatile
    }

    pub fn has_qualifiers_of(&self, other: &Self) -> bool {
        self.is_const >= other.is_const && self.is_volatile >= other.is_volatile
    }

    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        self.same_qualifiers_as(other) && self.is_compatible_ignoring_qualifiers(sema, other)
    }

    pub fn is_compatible_ignoring_qualifiers(&self, sema: &Sema, other: &Self) -> bool {
        if self.id == other.id {
            return true;
        }
        match (self.id.resolve(sema), other.id.resolve(sema)) {
            // 6.5.4.3 For two function types to be compatible, both shall specify compatible return types. Moreover...
            (ResolvedType::Function { ret: r1, params: p1 }, ResolvedType::Function { ret: r2, params: p2 }) => {
                r1.is_compatible(sema, r2) && p1.is_compatible(sema, p2)
            }
            // 6.5.4.2 For two array types to be compatible, both shall have compatible element types, and
            // if both size specifiers are present, they shall have the same value.
            (ResolvedType::Array { elem: e1, len: l1 }, ResolvedType::Array { elem: e2, len: l2 }) => {
                e1.is_compatible(sema, e2) && (l1.is_none() || l2.is_none() || l1 == l2)
            }
            // 6.5.4.1 For two pointer types to be compatible, both shall be identically qualified and both
            // shall be pointers to compatible types.
            (ResolvedType::Pointer(l), ResolvedType::Pointer(r)) => l.is_compatible(sema, r),
            // 6.5.2.2 Each enumerated type shall be compatible with an integer type, the choice of type is
            // implementation-defined.
            (ResolvedType::Tag(id), ResolvedType::Int) | (ResolvedType::Int, ResolvedType::Tag(id)) => {
                (*id).resolve(sema).kind == Tag::Enum
            }
            _ => false,
        }
    }
}

impl QualifiedType {
    pub fn describe(&self, sema: &Sema, ctx: &Context) -> String {
        let mut out = String::new();
        if self.is_const {
            out.push_str("const ");
        }
        if self.is_volatile {
            out.push_str("volatile ")
        }
        match self.id.resolve(sema) {
            ResolvedType::Void => out.push_str("void"),
            ResolvedType::Char => out.push_str("char"),
            ResolvedType::SignedChar => out.push_str("signed char"),
            ResolvedType::UnsignedChar => out.push_str("unsigned char"),
            ResolvedType::Short => out.push_str("short"),
            ResolvedType::UnsignedShort => out.push_str("unsigned short"),
            ResolvedType::Int => out.push_str("int"),
            ResolvedType::UnsignedInt => out.push_str("unsigned int"),
            ResolvedType::Long => out.push_str("long"),
            ResolvedType::UnsignedLong => out.push_str("unsigned long"),
            ResolvedType::Float => out.push_str("float"),
            ResolvedType::Double => out.push_str("double"),
            ResolvedType::LongDouble => out.push_str("long double"),
            ResolvedType::Pointer(inner) => {
                match inner.id.resolve(sema) {
                    ResolvedType::Function { .. } | ResolvedType::Array { .. } => {
                        out.push_str(&format!("({})", inner.describe(sema, ctx)))
                    }
                    _ => out.push_str(&inner.describe(sema, ctx)),
                }
                out.push_str(" *");
            }
            &ResolvedType::Tag(id) => {
                let def = id.resolve(sema);
                let name = def.name.map_or("<anonymous>", |n| n.id.resolve(ctx).as_str());
                out.push_str(&format!("{} {}", def.kind(), name));
                if !def.is_complete {
                    out.push_str(" (incomplete)");
                }
            }
            ResolvedType::Array { elem, len } => {
                let (mut elem, mut len) = (elem, len);
                let mut dimensions = String::new();
                loop {
                    dimensions.push('[');
                    if let Some(len) = len {
                        dimensions.push_str(&len.to_string());
                    }
                    dimensions.push(']');
                    let ResolvedType::Array { elem: inner, len: size } = elem.id.resolve(sema) else { break };
                    (elem, len) = (inner, size);
                }
                out.push_str(&elem.describe(sema, ctx));
                out.push_str(&dimensions);
            }
            ResolvedType::Function { ret, params } => {
                out.push_str(&ret.describe(sema, ctx));
                out.push('(');
                if let ParamTypes::Prototype { params, is_variadic } = params {
                    let described: Vec<String> = params.iter().map(|param| param.describe(sema, ctx)).collect();
                    match described.is_empty() {
                        true => out.push_str("void"),
                        false => out.push_str(&described.join(", ")),
                    }
                    if *is_variadic {
                        out.push_str(", ...")
                    }
                }
                out.push(')');
            }
        }
        out
    }
}
