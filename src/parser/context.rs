use std::collections::HashSet;

use crate::ast::{DeclarationNode, DeclarationSpecifier, DeclaratorArena, Name};
use crate::ast::{EnumArena, ExpressionArena, StatementArena, Storage, StringArena, StringId, StructArena};
use crate::ast::{StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, UnionArena, VariantArena};
use crate::parser::Span;
use crate::semantic::{ResolvedTypeArena, SymbolArena};

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
    pub symbols: SymbolArena,
    pub resolved_type: ResolvedTypeArena,
}

#[derive(Debug)]
pub struct Context {
    pub typedefs: Vec<HashSet<StringId>>,
    pub arenas: Arenas,
    pub ast: TranslationUnitNode,
    pub file_name: String,
}

impl Context {
    pub fn new(file_name: String) -> Self {
        Self {
            typedefs: vec![HashSet::default()],
            arenas: Arenas::default(),
            ast: TranslationUnitNode::default(),
            file_name,
        }
    }

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
            Tag::Enum => panic!("only for structs and unions"),
        }
    }

    pub fn push_scope(&mut self) {
        self.typedefs.push(HashSet::default());
    }

    pub fn pop_scope(&mut self) {
        self.typedefs.pop();
    }

    pub fn add_symbol(&mut self, decl: &DeclarationNode) {
        if !decl
            .specifiers
            .contains(&DeclarationSpecifier::Storage(Storage::Typedef))
        {
            return;
        }
        for init_decl in &decl.init_declarators {
            let Some(name) = init_decl.declarator.ident(self) else {
                continue;
            };
            let i = name.id;
            self.typedefs.last_mut().map(|ts| ts.insert(i));
        }
    }
}
