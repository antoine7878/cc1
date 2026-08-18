use crate::parser::Span;
use crate::{ast_node, define_arena};

define_arena!(String, StringArena, StringId, names);

ast_node! {
    pub struct Name {
        pub id: StringId,
    }
}

impl StringArena {
    pub fn add(&mut self, name: String, span: Span) -> Name {
        Name::new(self.alloc(name), span)
    }
}
