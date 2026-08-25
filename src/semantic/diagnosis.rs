use std::fmt::{self, Display};
use std::io::{self, Write, stderr};

use crate::ast::Name;
use crate::parser::{Context, Span};
use crate::semantic::SymbolKind;
use crate::utils::{RED, YELLOW, report};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Warning => YELLOW,
            Severity::Error => RED,
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Diagnosis {
    BadArgumentsCount,
    SyntaxError,
    InvalidSizeof,
    UndeclaredIdentifier(Name),
    // 6.4
    NonConstantExpression,
    NonIntegerConstantExpression,
    CastToNonScalar,
    // 6.5
    EmptyDeclaration,
    MultipleStorageSpecifiers,
    BlockScopeNotExtern,
    InvalidTypeSpecifer,
    DuplicateTypeQualifers,
    /// 6.5.2.1 Structure and union specifiers
    NonIntBitFieldType,
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

impl Diagnosis {
    #[rustfmt::skip]
    pub fn severity(&self) -> Severity {
        match self {
            Diagnosis::BadArgumentsCount => Severity::Error,
            Diagnosis::SyntaxError => Severity::Error,
            Diagnosis::InvalidSizeof => Severity::Error,
            Diagnosis::UndeclaredIdentifier(_) => Severity::Error,
            Diagnosis::NonConstantExpression => Severity::Error,
            Diagnosis::NonIntegerConstantExpression => Severity::Error,
            Diagnosis::CastToNonScalar => Severity::Error,
            Diagnosis::EmptyDeclaration => Severity::Error,
            Diagnosis::MultipleStorageSpecifiers => Severity::Error,
            Diagnosis::BlockScopeNotExtern => Severity::Error,
            Diagnosis::InvalidTypeSpecifer => Severity::Error,
            Diagnosis::DuplicateTypeQualifers => Severity::Error,
            Diagnosis::NonIntBitFieldType => Severity::Error,
            Diagnosis::VariantBadValue => Severity::Error,
            Diagnosis::AutoRegisterExternal => Severity::Error,
            Diagnosis::NotFunctionTypeDeclarator => Severity::Error,
            Diagnosis::FunctionAutoExtern => Severity::Error,
            Diagnosis::ParameterOldStyleListLenMismatch => Severity::Error,
            Diagnosis::ParameterTypeListWithList => Severity::Error,
            Diagnosis::ParameterNotRegister => Severity::Error,
            Diagnosis::AbstractParameterDeclaration => Severity::Error,
            Diagnosis::MissingDeclarationInOldStyle => Severity::Error,
            Diagnosis::DuplicateParameterName => Severity::Error,
            Diagnosis::MissingParameterInOldStyle => Severity::Error,
            Diagnosis::TypedefInOldStyle => Severity::Error,
            Diagnosis::LabelOutsideFunction => Severity::Error,
            Diagnosis::DuplicateDeclaration(_, _) => Severity::Error,
        }
    }
}

impl DiagnosisNode {
    pub fn new(inner: Diagnosis, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn severity(&self) -> Severity {
        self.inner.severity()
    }

    pub fn is_error(&self) -> bool {
        self.severity() == Severity::Error
    }

    pub fn print(&self, ctx: &Context) -> io::Result<()> {
        self.write(&mut stderr(), ctx)
    }

    pub fn write<W: Write>(&self, w: &mut W, ctx: &Context) -> io::Result<()> {
        report(w, ctx, self.span, self.severity(), self.message(ctx))
    }

    #[rustfmt::skip]
    fn message(&self, ctx: &Context) -> String {

        match &self.inner {
            Diagnosis::BadArgumentsCount => "wrong argument count".to_string(),
            Diagnosis::SyntaxError => "syntax error".to_string(),
            Diagnosis::InvalidSizeof => "invalid application of sizeof".to_string(),
            Diagnosis::UndeclaredIdentifier(name) => format!("Use of undeclared identifier '{}'", name.id.resolve(ctx)),
            Diagnosis::NonConstantExpression => "Non constant expression".to_string(),
            Diagnosis::NonIntegerConstantExpression => "Non integer constant expression".to_string(),
            Diagnosis::CastToNonScalar => "Conversion to non scalar type requested".to_string(),
            Diagnosis::VariantBadValue => "Variant value should be in int range".to_string(),
            Diagnosis::EmptyDeclaration => "Declaration declares nothing".to_string(),
            Diagnosis::MultipleStorageSpecifiers => "Multiple storage class declaration".to_string(),
            Diagnosis::BlockScopeNotExtern => "Function in block not declared as extern".to_string(),
            Diagnosis::InvalidTypeSpecifer => "Invalid type specifer or combination thereof".to_string(),
            Diagnosis::DuplicateTypeQualifers => "Duplicate type qualifers".to_string(),
            Diagnosis::AutoRegisterExternal => "External declaration auto of register".to_string(),
            Diagnosis::NotFunctionTypeDeclarator => "Declarator shall be function type".to_string(),
            Diagnosis::FunctionAutoExtern => "Function storage shall be auto or extern".to_string(),
            Diagnosis::ParameterOldStyleListLenMismatch => "Old style parameter function declaration shall be followed by a declaration list".to_string(),
            Diagnosis::ParameterTypeListWithList => "Parameter style function declration shall not be followed by a declaration list".to_string(),
            Diagnosis::ParameterNotRegister => "Parameter shall only by declared with register storage".to_string(),
            Diagnosis::AbstractParameterDeclaration => "Absctract declaration in old style function".to_string(),
            Diagnosis::MissingDeclarationInOldStyle => "Missing argument declaration in old style function".to_string(),
            Diagnosis::MissingParameterInOldStyle => "Missing parameter".to_string(),
            Diagnosis::DuplicateParameterName => "Duplicate paramter identifier".to_string(),
            Diagnosis::TypedefInOldStyle => "Typedef unsed in old style function".to_string(),
            Diagnosis::LabelOutsideFunction => "Label outside function".to_string(),
            Diagnosis::DuplicateDeclaration(kind, name) => format!("duplicate declaration of {} `{}'", kind, name.id.resolve(ctx)),
            Diagnosis::NonIntBitFieldType => "Bit-field has non-integral type".to_string(),
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
