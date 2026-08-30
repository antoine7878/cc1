use crate::ast::{Tag, TypeSpecifier};
use crate::define_interner;
use crate::semantic::{ParamTypes, Sema, TagDefArena, TagDefId};

define_interner!(ResolvedType, ResolvedTypeArena, ResolvedTypeId, crate::semantic::Sema, types);

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

// Two types have comparihle type if their types are the same. Additional rules for determining
// whether two types are compatible are described in 6.5., 3 for type specifiers, in 65.3 for type
// qualifiers, and in 6.5.4 for declarators.” Moreover. two structure, union. or enumeration types
// declared in separate translation units are compatible if they have the same number of members.
// the same member names. and compatible member types: for two structures. the members shall be
// in the same order: for two structures or unions, the bit-fields shall have the same widths: for two
// enumerations. the members shall have the same values

impl ResolvedType {
    // 6.1.2.5 An array type of unknown size is an incomplete type. A structure or union type of
    // unknown content is an incomplete type.
    pub fn is_complete(&self, tags: &TagDefArena) -> bool {
        match self {
            ResolvedType::Void => false,
            ResolvedType::Array { len, .. } => len.is_some(),
            ResolvedType::Tag(id) => tags.get(*id).is_complete,
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

    // 6.1.2.5 Integral and floating types are collectively called arithmetic types.
    pub fn is_arithmetic(&self, sema: &Sema) -> bool {
        self.is_integral(sema) || self.is_floating()
    }

    // 6.1.2.5 The type char, the signed and unsigned integer types, and the enumerated types
    // are collectively called integral types.
    pub fn is_integral(&self, sema: &Sema) -> bool {
        match self {
            ResolvedType::Tag(id) => sema.tags.get(*id).kind == Tag::Enum,
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
}

impl Builtins {
    pub fn new(types: &mut ResolvedTypeArena) -> Self {
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

    pub fn same_qualifiers(&self, other: &Self) -> bool {
        self.is_const == other.is_const && self.is_volatile == other.is_volatile
    }

    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        // 6.5.3 For two qualified types to be compatible, both shall have the identically qualified
        // version of a compatible type; the order of type qualifiers within a list of specifiers or
        // qualifiers does not affect the specified type.
        if !self.same_qualifiers(other) {
            return false;
        }
        if self.id == other.id {
            return true;
        }
        match (sema.types.get(self.id), sema.types.get(other.id)) {
            // 6.5.4.3 For two function types to be compatible, both shall specify compatible return types.
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
                sema.tags.get(*id).kind == Tag::Enum
            }
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct TypeSpecifierCounter {
    pub void: u8,
    pub char: u8,
    pub short: u8,
    pub int: u8,
    pub long: u8,
    pub float: u8,
    pub double: u8,
    pub signed: u8,
    pub unsigned: u8,
}

impl TypeSpecifierCounter {
    pub fn count(types: &[&TypeSpecifier]) -> Option<[u8; 9]> {
        let mut c = Self::default();
        for ty in types {
            let a = match ty {
                TypeSpecifier::Void => &mut c.void,
                TypeSpecifier::Char => &mut c.char,
                TypeSpecifier::Short => &mut c.short,
                TypeSpecifier::Int => &mut c.int,
                TypeSpecifier::Long => &mut c.long,
                TypeSpecifier::Float => &mut c.float,
                TypeSpecifier::Double => &mut c.double,
                TypeSpecifier::Signed => &mut c.signed,
                TypeSpecifier::Unsigned => &mut c.unsigned,
                TypeSpecifier::Struct(_) => return None,
                TypeSpecifier::Union(_) => return None,
                TypeSpecifier::Enum(_) => return None,
                TypeSpecifier::TypedefName(_) => return None,
            };
            if *a > 0 {
                return None;
            };
            *a += 1;
        }
        Some([
            c.signed, c.unsigned, c.void, c.char, c.short, c.int, c.long, c.float, c.double,
        ])
    }
}
