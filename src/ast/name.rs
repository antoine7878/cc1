use std::fmt;

use crate::ast::Node;
use crate::define_interner;
use crate::parser::Span;
use crate::utils::{BLUE, RESET};

define_interner!(String, StringArena, StringId, crate::parser::Arenas, names);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Name {
    pub span: Span,
    pub id: StringId,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{BLUE}Name{RESET} {}", self.span)
    }
}

impl Name {
    pub fn new(id: StringId, span: Span) -> Self {
        Self { id, span }
    }
}
impl Node for Name {
    fn span(&self) -> Span {
        self.span
    }
}

impl StringArena {
    pub fn add(&mut self, name: String, span: Span) -> Name {
        Name::new(self.alloc(name), span)
    }
}
