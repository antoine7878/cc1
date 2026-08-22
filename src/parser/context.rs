use std::collections::HashMap;

use crate::ast::{DeclarationSpecifier, DeclaratorArena, Name, Storage};
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
    in_typedef_stack: Vec<bool>,
    struct_depth: usize,
    type_name_id: usize,
    identifier_id: usize,
    type_name_ok: bool,
    identifier_ok: bool,
    stashed: Option<HashMap<StringId, SymbolKind>>,
}

impl Context {
    pub fn new(file_name: String) -> Self {
        Self {
            typedefs: vec![HashMap::default()],
            arenas: Arenas::default(),
            ast: TranslationUnitNode::default(),
            in_typedef: false,
            in_typedef_stack: Vec::new(),
            struct_depth: 0,
            type_name_id: YYToken::id_of("TYPE_NAME").expect("TYPE_NAME token"),
            identifier_id: YYToken::id_of("IDENTIFIER").expect("IDENTIFIER token"),
            type_name_ok: false,
            identifier_ok: true,
            stashed: None,
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
        self.in_typedef_stack.push(self.in_typedef);
    }

    pub fn pop_scope(&mut self) {
        self.stashed = self.typedefs.pop();
        self.in_typedef = self.in_typedef_stack.pop().unwrap_or(false);
    }

    pub fn unstash_scope(&mut self) {
        self.push_scope();
        if let Some(stashed) = self.stashed.take() {
            *self.typedefs.last_mut().unwrap() = stashed;
        }
    }

    pub fn note_specifiers(&mut self, specs: Vec<DeclarationSpecifier>) -> Vec<DeclarationSpecifier> {
        self.in_typedef = specs.contains(&DeclarationSpecifier::Storage(Storage::Typedef));
        specs
    }

    pub fn enter_struct(&mut self) {
        self.struct_depth += 1;
    }

    pub fn exit_struct(&mut self) {
        self.struct_depth -= 1;
    }

    pub fn add_symbol(&mut self, id: StringId) {
        if self.struct_depth > 0 {
            return;
        }
        let kind = if self.in_typedef { SymbolKind::Typedef } else { SymbolKind::Variable };
        self.typedefs.last_mut().map(|ts| ts.insert(id, kind));
    }

    pub fn feedback(&mut self, accepts: &dyn Fn(usize) -> bool) {
        self.type_name_ok = accepts(self.type_name_id);
        self.identifier_ok = accepts(self.identifier_id);
    }

    pub fn check_type(&mut self, name: Name) -> YYToken {
        match (self.type_name_ok, self.identifier_ok) {
            (false, _) => YYToken::IDENTIFIER(name),
            (true, false) => YYToken::TYPE_NAME(name),
            (true, true) => match self.typedefs.iter().rev().find_map(|ty| ty.get(&name.id)) {
                Some(SymbolKind::Typedef) => YYToken::TYPE_NAME(name),
                _ => YYToken::IDENTIFIER(name),
            },
        }
    }

    // pub fn concat_strings(&mut self, names: Vec<String>) -> Name {
    //     // let s1 = n1.id.resolve(self);
    //     // let s2 = n2.id.resolve(self);
    //     let s = String::from(s1) + s2;
    //     let span = Span::merge(n1.span, n2.span);
    //     self.arenas.names.add(s, span)
    // }
}
