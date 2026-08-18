use std::io::{self, Write, stdout};

use crate::{
    ast::Name,
    parser::{Context, Span},
};

#[derive(Debug)]
pub struct Diag<T> {
    pub res: T,
    pub diagnosis: Option<DiagnosisInner>,
}

impl<T> Diag<T> {
    pub fn new(res: T, diagnosis: Option<DiagnosisInner>) -> Self {
        Self { res, diagnosis }
    }

    pub fn with_diag(res: T, diagnosis: DiagnosisInner) -> Self {
        Self::new(res, Some(diagnosis))
    }

    pub fn res(res: T) -> Self {
        Self::new(res, None)
    }
}

impl<T> Diag<Option<T>> {
    pub fn diag_none(diagnosis: DiagnosisInner) -> Self {
        Self::new(None, Some(diagnosis))
    }

    pub fn diag_some(res: T, diagnosis: DiagnosisInner) -> Self {
        Self::new(Some(res), Some(diagnosis))
    }

    pub fn res_none() -> Self {
        Self::new(None, None)
    }

    pub fn res_some(res: T) -> Self {
        Self::new(Some(res), None)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Diagnosis {
    span: Span,
    inner: DiagnosisInner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DiagnosisInner {
    UndeclaredIdentifier(Name),
    MultipleStorageSpecifiers,
    BlockScopeNotExtern,
    InvalidTypeSpecifer,
    DuplicateTypeQualifers,
}

impl Diagnosis {
    pub fn new(inner: DiagnosisInner, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn print(&self, ctx: &Context) -> io::Result<()> {
        self.write(&mut stdout(), ctx)
    }

    #[rustfmt::skip]
    pub fn write<W: Write>(&self, w: &mut W, ctx: &Context) -> io::Result<()> {
        write!(w, "{}:{}:{} ", ctx.file_name, self.span.start.line, self.span.start.col)?;
        match &self.inner {
            DiagnosisInner::UndeclaredIdentifier(name) => writeln!(w, "Use of undeclared identifier '{}'", name.id.resolve(&ctx.arenas)),
            DiagnosisInner::MultipleStorageSpecifiers => writeln!(w, "Multiple storage class declaration"),
            DiagnosisInner::BlockScopeNotExtern => writeln!(w, "Function in block not declared as extern"),
            DiagnosisInner::InvalidTypeSpecifer => writeln!(w, "Invalid type specifer or combination thereof"),
            DiagnosisInner::DuplicateTypeQualifers => writeln!(w, "Duplicate type qualifers"),
       } }
}
