use crate::symbol::{NameArena, NameId, SymbolTable, TagKind};
use crate::tag::{EnumArena, Field, StructArena, UnionArena, VariantArena};
use crate::types::TypeArena;

#[derive(Debug, Default)]
pub struct Arenas {
    pub symbols: SymbolTable,
    pub names: NameArena,
    pub types: TypeArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
}

#[derive(Debug, Default)]
#[allow(unused)]
pub struct Context {
    pub symbols: SymbolTable,
    pub arenas: Arenas,
}
