use std::collections::HashMap;

use crate::ast::{DeclaratorArena, Name};
use crate::ast::{EnumArena, ExpressionArena, StatementArena, StringArena, StringId, StructArena};
use crate::ast::{StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, UnionArena, VariantArena};
use crate::parser::{Span, YYToken};
use crate::semantic::{ResolvedTypeArena, SymbolArena, SymbolKind};

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
    pub typedefs: Vec<HashMap<StringId, SymbolKind>>,
    pub arenas: Arenas,
    pub ast: TranslationUnitNode,
    pub file_name: String,
    pub in_typedef: bool,
}

impl Context {
    pub fn new(file_name: String) -> Self {
        Self {
            typedefs: vec![HashMap::default()],
            arenas: Arenas::default(),
            ast: TranslationUnitNode::default(),
            in_typedef: false,
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
        self.typedefs.push(HashMap::default());
    }

    pub fn pop_scope(&mut self) {
        self.typedefs.pop();
    }

    pub fn add_symbol(&mut self, id: StringId) {
        let kind = if self.in_typedef { SymbolKind::Typedef } else { SymbolKind::Variable };
        self.typedefs.last_mut().map(|ts| ts.insert(id, kind));
    }

    pub fn check_type(&self, name: Name) -> YYToken {
        match self.typedefs.iter().rev().filter_map(|ty| ty.get(&name.id)).next() {
            Some(SymbolKind::Typedef) => YYToken::TYPE_NAME(name),
            Some(SymbolKind::Variable) | None => YYToken::IDENTIFIER(name),
            _ => unreachable!(),
        }
    }
}
