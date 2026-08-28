use crate::{
    ast::TypeSpecifier,
    define_interner,
    semantic::{ParamTypes, TagDefId},
};

define_interner!(ResolvedType, ResolvedTypeArena, ResolvedTypeId, sema.types);

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
        let id = match self.get(param.ty).clone() {
            ResolvedType::Array { elem, .. } => self.pointer(elem),
            ResolvedType::Function { .. } => self.pointer(QualifiedType::new(param.ty, false, false)),
            _ => return param,
        };
        QualifiedType::new(id, param.is_const, param.is_volatile)
    }

    fn parameter(&mut self, param: QualifiedType) -> QualifiedType {
        let param = self.adjust_parameter(param);
        QualifiedType::new(param.ty, false, false)
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
