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
