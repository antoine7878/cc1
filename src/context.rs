use std::io::Read;

use crate::ast::NodeArena;
use crate::lexer::YYLex;
use crate::parser::Yacc;
use crate::symbol::{NameArena, SymbolTable};
use crate::tag::{EnumArena, StructArena, UnionArena, VariantArena};
use crate::types::TypeArena;

#[derive(Debug, Default)]
pub struct Arenas {
    pub names: NameArena,
    pub types: TypeArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
    pub nodes: NodeArena,
}

#[derive(Debug, Default)]
pub struct Context {
    pub symbols: SymbolTable,
    pub arenas: Arenas,
}

pub trait ContextAccess {
    fn ctx(&mut self) -> &mut Context;
    fn names(&mut self) -> &mut NameArena {
        &mut self.ctx().arenas.names
    }

    fn types(&mut self) -> &mut TypeArena {
        &mut self.ctx().arenas.types
    }

    fn structs(&mut self) -> &mut StructArena {
        &mut self.ctx().arenas.structs
    }

    fn enums(&mut self) -> &mut EnumArena {
        &mut self.ctx().arenas.enums
    }

    fn unions(&mut self) -> &mut UnionArena {
        &mut self.ctx().arenas.unions
    }

    fn variants(&mut self) -> &mut VariantArena {
        &mut self.ctx().arenas.variants
    }

    fn nodes(&mut self) -> &mut NodeArena {
        &mut self.ctx().arenas.nodes
    }
}

impl<R: Read> ContextAccess for YYLex<R> {
    fn ctx(&mut self) -> &mut Context {
        &mut self.ctx
    }
}

impl<R: Read> ContextAccess for Yacc<R> {
    fn ctx(&mut self) -> &mut Context {
        &mut self.lexer.ctx
    }
}
