use std::collections::HashSet;

use crate::ast::{DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorArena, DeclaratorNode, Name};
use crate::ast::{EnumArena, ExpressionArena, StatementArena, Storage, StringArena, StringId, StructArena};
use crate::ast::{StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, UnionArena, VariantArena};
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
    pub typedefs: HashSet<StringId>,
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
            Tag::Enum => panic!("only for structs and unions"),
        }
    }

    pub fn add_symbol(&mut self, decl: &DeclarationNode) {
        if !decl
            .specifiers
            .contains(&DeclarationSpecifier::Storage(Storage::Typedef))
        {
            return;
        }
        for init_decl in &decl.init_declarators {
            let Some(name) = self.declartor_name(&init_decl.declarator) else {
                continue;
            };
            self.typedefs.insert(name.id);
        }
    }

    pub fn declartor_name(&self, decl: &DeclaratorNode) -> Option<&Name> {
        match self.arenas.declarators.get(decl.id) {
            Declarator::Ident(name) => Some(name),
            Declarator::Pointer { inner: Some(d), .. } => self.declartor_name(d),
            Declarator::Array { declarator, .. } => self.declartor_name(declarator),
            Declarator::Function { declarator, .. } => self.declartor_name(declarator),
            _ => None,
        }
    }
}
