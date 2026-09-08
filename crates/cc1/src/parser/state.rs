use std::collections::HashMap;

use crate::ast::{DeclarationSpecifier, Name, Storage, StringId};
use crate::parser::YYToken;
use crate::semantic::SymbolKind;

#[derive(Debug)]
pub struct ParseState {
    typedefs: Vec<HashMap<StringId, SymbolKind>>,
    in_typedef: bool,
    in_typedef_stack: Vec<bool>,
    struct_depth: usize,
    stashed: Option<HashMap<StringId, SymbolKind>>,
    type_name_id: usize,
    identifier_id: usize,
    type_name_ok: bool,
    identifier_ok: bool,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            typedefs: vec![HashMap::default()],
            in_typedef: false,
            in_typedef_stack: Vec::new(),
            struct_depth: 0,
            stashed: None,
            type_name_id: YYToken::id_of("TYPE_NAME").expect("TYPE_NAME token"),
            identifier_id: YYToken::id_of("IDENTIFIER").expect("IDENTIFIER token"),
            type_name_ok: false,
            identifier_ok: true,
        }
    }
}

impl ParseState {
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

    pub fn recover_to_file_scope(&mut self) {
        self.typedefs.truncate(1);
        self.in_typedef_stack.clear();
        self.in_typedef = false;
        self.struct_depth = 0;
        self.stashed = None;
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
}
