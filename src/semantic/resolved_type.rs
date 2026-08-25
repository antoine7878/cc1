use crate::{ast::TypeSpecifier, define_interner, semantic::TagDefId};

define_interner!(ResolvedType, ResolvedTypeArena, ResolvedTypeId, resolved_type);

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
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
    Array { elem: QualifiedType, len: Option<u32> },
    Pointer(QualifiedType),
    Tag(TagDefId),
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub struct QualifiedType {
    pub ty: ResolvedTypeId,
    pub is_const: bool,
    pub is_volatile: bool,
}

impl ResolvedType {
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

impl ResolvedTypeArena {
    pub fn int(&mut self) -> ResolvedTypeId {
        self.alloc(ResolvedType::Int)
    }

    pub fn pointer(&mut self, inner: QualifiedType) -> ResolvedTypeId {
        self.alloc(ResolvedType::Pointer(inner))
    }

    pub fn tag(&mut self, id: TagDefId) -> ResolvedTypeId {
        self.alloc(ResolvedType::Tag(id))
    }
}

impl QualifiedType {
    pub fn new(ty: ResolvedTypeId, is_const: bool, is_volatile: bool) -> Self {
        QualifiedType {
            ty,
            is_const,
            is_volatile,
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
