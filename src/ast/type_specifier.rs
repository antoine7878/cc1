use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

use crate::ast::{EnumId, Name, StructId, UnionId};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DeclarationSpecifier {
    Type(TypeSpecifier),
    Qualifier(Qualifier),
    Storage(Storage),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TypeSpecifier {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    Struct(StructId),
    Union(UnionId),
    Enum(EnumId),
    TypedefName(Name),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Qualifier {
    Const,
    Volatile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Storage {
    Auto,
    Static,
    Extern,
    Typedef,
    Register,
    ThreadLocal,
}

impl Display for TypeSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TypeSpecifier::Void => "void",
            TypeSpecifier::Char => "char",
            TypeSpecifier::Short => "short",
            TypeSpecifier::Int => "int",
            TypeSpecifier::Long => "long",
            TypeSpecifier::Float => "float",
            TypeSpecifier::Double => "double",
            TypeSpecifier::Signed => "signed",
            TypeSpecifier::Unsigned => "unsigned",
            TypeSpecifier::Struct(_) => "struct",
            TypeSpecifier::Union(_) => "union",
            TypeSpecifier::Enum(_) => "enum",
            TypeSpecifier::TypedefName(_) => "typedef name",
        };
        write!(f, "{}", s)
    }
}

impl Display for Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Qualifier::Const => "const",
            Qualifier::Volatile => "volatile",
        };
        write!(f, "{}", s)
    }
}

impl Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Storage::Auto => "auto",
            Storage::Static => "static",
            Storage::Extern => "extern",
            Storage::Typedef => "typedef",
            Storage::Register => "register",
            Storage::ThreadLocal => "thread_local",
        };
        write!(f, "{}", s)
    }
}

// impl TypeArena {
//     pub fn void(&mut self) -> TypeId {
//         self.alloc(Type::Void)
//     }
//
//     pub fn char(&mut self) -> TypeId {
//         self.alloc(Type::Char)
//     }
//
//     pub fn short(&mut self) -> TypeId {
//         self.alloc(Type::Short)
//     }
//     pub fn int(&mut self) -> TypeId {
//         self.alloc(Type::Int)
//     }
//
//     pub fn long(&mut self) -> TypeId {
//         self.alloc(Type::Long)
//     }
//
//     pub fn signed(&mut self) -> TypeId {
//         self.alloc(Type::Signed)
//     }
//
//     pub fn unsigned(&mut self) -> TypeId {
//         self.alloc(Type::Unsigned)
//     }
//
//     pub fn float(&mut self) -> TypeId {
//         self.alloc(Type::Float)
//     }
//
//     pub fn double(&mut self) -> TypeId {
//         self.alloc(Type::Double)
//     }
//
//     pub fn longdouble(&mut self) -> TypeId {
//         self.alloc(Type::LongDouble)
//     }
//
//     pub fn pointer(&mut self, base: TypeId) -> TypeId {
//         self.alloc(Type::Pointer(base))
//     }
//
//     pub fn array(&mut self, element: TypeId, length: Option<u64>) -> TypeId {
//         self.alloc(Type::Array { element, length })
//     }
//
//     pub fn function(&mut self, return_type: TypeId, params: Vec<TypeId>, variadic: bool) -> TypeId {
//         self.alloc(Type::Function {
//             return_type,
//             params,
//             variadic,
//         })
//     }
//
//     pub fn struct_(&mut self, base: StructId) -> TypeId {
//         self.alloc(Type::Struct(base))
//     }
//
//     pub fn union_(&mut self, base: UnionId) -> TypeId {
//         self.alloc(Type::Union(base))
//     }
//
//     pub fn enum_(&mut self, base: EnumId) -> TypeId {
//         self.alloc(Type::Enum(base))
//     }
//
//     // pub fn qualified(&mut self, base: TypeId, qualifiers: BTreeSet<Qualifier>) -> TypeId {
//     //     self.alloc(Type::Qualified { base, qualifiers })
//     // }
// }
