use crate::arena::{Arena, ArenaId};
use crate::define_arena;
use crate::parser::Span;

define_arena!(String, StringArena, StringId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Name {
    pub span: Span,
    pub id: StringId,
}

impl StringArena {
    pub fn name(&mut self, name: String, span: Span) -> Name {
        Name {
            id: self.alloc(name),
            span,
        }
    }
}
