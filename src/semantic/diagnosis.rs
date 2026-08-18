use std::io::{self, Write, stdout};

use crate::ast::Name;
use crate::ast_node;
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::DiagnosisInner::UndeclaredIdentifier;

ast_node! {
    pub struct Diagnosis {
        inner: DiagnosisInner,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DiagnosisInner {
    UndeclaredIdentifier(Name),
}

impl Diagnosis {
    pub fn undeclared_identifier(name: Name, span: Span) -> Self {
        Self::new(UndeclaredIdentifier(name), span)
    }

    pub fn print(&self, ctx: &Context) -> io::Result<()> {
        self.write(&mut stdout(), ctx)
    }

    pub fn write<W: Write>(&self, w: &mut W, ctx: &Context) -> io::Result<()> {
        write!(w, "{}:{}:{} ", ctx.file_name, self.span.start.line, self.span.start.col)?;
        match &self.inner {
            DiagnosisInner::UndeclaredIdentifier(name) => {
                writeln!(w, "Use of undeclared identifier '{}'", name.id.resolve(&ctx.arenas))
            }
        }
    }
}
