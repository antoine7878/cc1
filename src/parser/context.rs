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
    in_typedef_stack: Vec<bool>,
    pub struct_depth: usize,
    pub saw_type_spec: bool,
    pub pending_tag: bool,
    brace_stack: Vec<bool>,
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
            saw_type_spec: false,
            pending_tag: false,
            brace_stack: Vec::new(),
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

    pub fn add_symbol(&mut self, id: StringId) {
        if self.struct_depth > 0 {
            return;
        }
        let kind = if self.in_typedef { SymbolKind::Typedef } else { SymbolKind::Variable };
        self.typedefs.last_mut().map(|ts| ts.insert(id, kind));
    }

    pub fn check_type(&mut self, name: Name) -> YYToken {
        if self.pending_tag {
            // struct/union/enum tag name: `struct S` is itself a complete type specifier
            self.saw_type_spec = true;
            return YYToken::IDENTIFIER(name);
        }
        if self.saw_type_spec {
            // a type specifier was already seen: this identifier is a declarator name
            return YYToken::IDENTIFIER(name);
        }
        match self.typedefs.iter().rev().filter_map(|ty| ty.get(&name.id)).next() {
            Some(SymbolKind::Typedef) => {
                self.saw_type_spec = true;
                YYToken::TYPE_NAME(name)
            }
            Some(SymbolKind::Variable) | None => YYToken::IDENTIFIER(name),
            _ => unreachable!(),
        }
    }

    pub fn enter_brace(&mut self) {
        self.brace_stack.push(self.pending_tag);
        self.pending_tag = false;
        self.saw_type_spec = false;
    }

    pub fn exit_brace(&mut self) {
        self.saw_type_spec = self.brace_stack.pop().unwrap_or(false);
    }

    pub fn clear_type_spec(&mut self) {
        self.saw_type_spec = false;
        self.pending_tag = false;
    }
}
