use crate::ast::{
    DeclaratorArena, Name, StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, UnionArena, VariantArena,
};
use crate::ast::{EnumArena, ExpressionArena, StatementArena, StringArena, StructArena};
use crate::parser::Span;

#[derive(Debug, Default)]
pub struct Arenas {
    pub names: StringArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
    pub expressions: ExpressionArena,
    pub declarators: DeclaratorArena,
    pub statements: StatementArena,
}

#[derive(Debug, Default)]
pub struct Context {
    // pub symbols: SymbolTable,
    pub arenas: Arenas,
    pub ast: TranslationUnitNode,
}

impl Context {
    pub fn struct_or_union(
        &mut self,
        tag: Tag,
        name: Option<Name>,
        fields: Vec<StructDeclaration>,
        span: Span,
    ) -> TypeSpecifier {
        match tag {
            Tag::Struct => TypeSpecifier::Struct(self.arenas.structs.add(name, fields, span)),
            Tag::Union => TypeSpecifier::Union(self.arenas.unions.add(name, fields, span)),
            _ => unimplemented!(),
        }
    }
}

// pub trait ContextAccess {
//     fn ctx(&mut self) -> &mut Context;
//     fn names(&mut self) -> &mut StringArena {
//         &mut self.ctx().arenas.names
//     }
//
//     fn structs(&mut self) -> &mut StructArena {
//         &mut self.ctx().arenas.structs
//     }
//
//     fn enums(&mut self) -> &mut EnumArena {
//         &mut self.ctx().arenas.enums
//     }
//
//     fn unions(&mut self) -> &mut UnionArena {
//         &mut self.ctx().arenas.unions
//     }
//
//     fn variants(&mut self) -> &mut VariantArena {
//         &mut self.ctx().arenas.variants
//     }
//
//     fn expressions(&mut self) -> &mut ExpressionArena {
//         &mut self.ctx().arenas.expressions
//     }
// }
//
// impl<R: Read> ContextAccess for YYLex<R> {
//     fn ctx(&mut self) -> &mut Context {
//         &mut self.ctx
//     }
// }
