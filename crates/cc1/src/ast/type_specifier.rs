use std::fmt::Debug;

use crate::ast::{EnumId, Name, StructId, UnionId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationSpecifier {
    Type(TypeSpecifier),
    Qualifier(Qualifier),
    Storage(Storage),
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qualifier {
    Const,
    Volatile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Storage {
    Typedef,
    Extern,
    Static,
    Auto,
    Register,
}
