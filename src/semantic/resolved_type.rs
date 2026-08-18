use crate::ast::{EnumId, Name, StructId, TypeSpecifier, UnionId};

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
            c.signed, c.unsigned, c.char, c.short, c.short, c.int, c.long, c.float, c.double,
        ])
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd)]
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
    Struct(StructId),
    Union(UnionId),
    Enum(EnumId),
    Typedef(Name),
}
