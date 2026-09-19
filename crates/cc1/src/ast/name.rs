use std::fmt;

use libft::{BLUE, RESET, Span};

use crate::ast::Node;
use crate::define_interner;

define_interner!(String, NameInterner, NameId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Name {
    pub span: Span,
    pub id: NameId,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{BLUE}Name{RESET} {}", self.span)
    }
}

impl Name {
    pub fn new(id: NameId, span: Span) -> Self {
        Self { id, span }
    }
}
impl Node for Name {
    fn span(&self) -> Span {
        self.span
    }
}

impl NameInterner {
    pub fn add(&mut self, name: String, span: Span) -> Name {
        Name::new(self.intern(name), span)
    }
}
