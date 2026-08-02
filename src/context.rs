use crate::ast::{DeclaratorArena, UnionArena, VariantArena};
use crate::ast::{EnumArena, ExpressionArena, StringArena, StructArena};
use crate::parser::YYLex;
use std::io::Read;

#[derive(Debug, Default)]
pub struct Arenas {
    pub names: StringArena,
    // pub types: TypeArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
    pub expressions: ExpressionArena,
    pub declarators: DeclaratorArena,
}

#[derive(Debug, Default)]
pub struct Context {
    // pub symbols: SymbolTable,
    pub arenas: Arenas,
}

pub trait ContextAccess {
    fn ctx(&mut self) -> &mut Context;
    fn names(&mut self) -> &mut StringArena {
        &mut self.ctx().arenas.names
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

    fn expressions(&mut self) -> &mut ExpressionArena {
        &mut self.ctx().arenas.expressions
    }
}

impl<R: Read> ContextAccess for YYLex<R> {
    fn ctx(&mut self) -> &mut Context {
        &mut self.ctx
    }
}

// impl<R: Read> ContextAccess for Yacc<R> {
//     fn ctx(&mut self) -> &mut Context {
//         &mut self.lexer.ctx
//     }
// }
