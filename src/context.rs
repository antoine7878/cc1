use crate::symbol::{NameArena, SymbolTable};
use crate::tag::{EnumArena, StructArena, VariantArena};
use crate::types::TypeArena;

#[derive(Debug, Default)]
pub struct Arenas {
    pub symbols: SymbolTable,
    pub names: NameArena,
    pub types: TypeArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: EnumArena,
    pub variants: VariantArena,
}

#[derive(Debug, Default)]
#[allow(unused)]
pub struct Context {
    symbols: SymbolTable,
    arenas: Arenas,
}
