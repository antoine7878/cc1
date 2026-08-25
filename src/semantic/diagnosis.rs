use std::io::{self, Write, stderr};

use crate::ast::Name;
use crate::parser::{Context, Span};
use crate::semantic::SymbolKind;

#[derive(Clone, Copy, Debug)]
pub enum Diagnosis {
    InvalidSizeof,
    UndeclaredIdentifier(Name),
    // 6.4
    NonConstantExpression,
    NonIntegerConstantExpression,
    CastToNonScalar,
    // 6.5
    MultipleStorageSpecifiers,
    BlockScopeNotExtern,
    InvalidTypeSpecifer,
    DuplicateTypeQualifers,
    /// 6.5.2.1 Structure and union specifiers
    NonIntegerBitFieldType,
    // 6.5.2.2 Enumeration specifiers
    VariantBadValue,
    // 6.7
    AutoRegisterExternal,
    // 6.7.1
    NotFunctionTypeDeclarator,
    FunctionAutoExtern,
    ParameterOldStyleListLenMismatch,
    ParameterTypeListWithList,
    ParameterNotRegister,
    AbstractParameterDeclaration,
    MissingDeclarationInOldStyle,
    DuplicateParameterName,
    MissingParameterInOldStyle,
    TypedefInOldStyle,

    LabelOutsideFunction,
    DuplicateDeclaration(SymbolKind, Name),
}

impl DiagnosisNode {
    pub fn new(inner: Diagnosis, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn print(&self, ctx: &Context) -> io::Result<()> {
        self.write(&mut stderr(), ctx)
    }

    #[rustfmt::skip]
    pub fn write<W: Write>(&self, w: &mut W, ctx: &Context) -> io::Result<()> {
        write!(w, "{}:{}:{} ", ctx.file_name, self.span.start.line, self.span.start.col)?;
        match &self.inner {
            Diagnosis::InvalidSizeof => writeln!(w, "invalid application of sizeof"),
            Diagnosis::UndeclaredIdentifier(name) => writeln!(w, "Use of undeclared identifier '{}'", name.id.resolve(ctx)),
            Diagnosis::NonConstantExpression => writeln!(w, "Non constant expression"),
            Diagnosis::NonIntegerConstantExpression => writeln!(w, "Non integer constant expression"),
            Diagnosis::CastToNonScalar => writeln!(w, "Conversion to non scalar type requested"),
            Diagnosis::VariantBadValue => writeln!(w, "Variant value should be in int range"),
            Diagnosis::MultipleStorageSpecifiers => writeln!(w, "Multiple storage class declaration"),
            Diagnosis::BlockScopeNotExtern => writeln!(w, "Function in block not declared as extern"),
            Diagnosis::InvalidTypeSpecifer => writeln!(w, "Invalid type specifer or combination thereof"),
            Diagnosis::DuplicateTypeQualifers => writeln!(w, "Duplicate type qualifers"),
            Diagnosis::AutoRegisterExternal => writeln!(w, "External declaration auto of register"),
            Diagnosis::NotFunctionTypeDeclarator => writeln!(w, "Declarator shall be function type"),
            Diagnosis::FunctionAutoExtern => writeln!(w, "Function storage shall be auto or extern"),
            Diagnosis::ParameterOldStyleListLenMismatch => writeln!(w, "Old style parameter function declaration shall be followed by a declaration list"),
            Diagnosis::ParameterTypeListWithList => writeln!(w, "Parameter style function declration shall not be followed by a declaration list"),
            Diagnosis::ParameterNotRegister => writeln!(w, "Parameter shall only by declared with register storage"),
            Diagnosis::AbstractParameterDeclaration => writeln!(w, "Absctract declaration in old style function"),
            Diagnosis::MissingDeclarationInOldStyle => writeln!(w, "Missing argument declaration in old style function"),
            Diagnosis::MissingParameterInOldStyle => writeln!(w, "Missing parameter"),
            Diagnosis::DuplicateParameterName => writeln!(w, "Duplicate paramter identifier"),
            Diagnosis::TypedefInOldStyle => writeln!(w, "Typedef unsed in old style function"),
            Diagnosis::LabelOutsideFunction => writeln!(w, "Label outside function"),
            Diagnosis::DuplicateDeclaration(kind, name) => writeln!(w, "duplicate declaration of {} `{}'", kind, name.id.resolve(ctx)),
            Diagnosis::NonIntegerBitFieldType => writeln!(w, "Bit-field has non-integral type" ),
            
        }
    }
}

pub trait DiagCollector {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode>;

    fn add_diag<T>(&mut self, diag: Diag<T>, span: &Span) -> T {
        if let Some(diagnosis) = diag.diagnosis {
            self.diagnosis().push(DiagnosisNode::new(diagnosis, *span));
        }
        diag.res
    }
}

#[derive(Debug)]
pub struct Diag<T> {
    pub res: T,
    pub diagnosis: Option<Diagnosis>,
}

impl Diag<()> {
    pub fn only_diag(diagnosis: Diagnosis) -> Self {
        Self::new((), Some(diagnosis))
    }
}

impl<T> Diag<T> {
    pub fn collect<C: DiagCollector>(self, collector: &mut C, span: &Span) -> T {
        collector.add_diag(self, span)
    }

    pub fn new(res: T, diagnosis: Option<Diagnosis>) -> Self {
        Self { res, diagnosis }
    }

    pub fn with_diag(res: T, diagnosis: Diagnosis) -> Self {
        Self::new(res, Some(diagnosis))
    }

    pub fn res(res: T) -> Self {
        Self::new(res, None)
    }
}

impl<T> Diag<Option<T>> {
    pub fn none_diag(diagnosis: Diagnosis) -> Self {
        Self::new(None, Some(diagnosis))
    }

    pub fn some_diag(res: T, diagnosis: Diagnosis) -> Self {
        Self::new(Some(res), Some(diagnosis))
    }

    pub fn none() -> Self {
        Self::new(None, None)
    }

    pub fn some(res: T) -> Self {
        Self::new(Some(res), None)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiagnosisNode {
    pub span: Span,
    pub inner: Diagnosis,
}
