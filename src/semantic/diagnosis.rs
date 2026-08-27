use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write, stderr};

use crate::ast::Name;
use crate::parser::{Context, Span};
use crate::semantic::SymbolKind;
use crate::utils::{RED, RESET, YELLOW};

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

pub const MAX_EXPECTED: usize = 5;

#[derive(Clone, Copy, Debug)]
pub struct ExpectedTokens {
    names: [&'static str; MAX_EXPECTED],
    len: usize,
}

const STRUCTURAL: [&str; 5] = ["';'", "','", "')'", "']'", "'}'"];

fn token_label(name: &str) -> &str {
    match name {
        "yyeof" => "end of file",
        name => name,
    }
}

impl ExpectedTokens {
    pub fn new(found: &'static str, names: &[&'static str]) -> Self {
        if names.len() <= MAX_EXPECTED {
            return Self::from_slice(names);
        }
        let structural: Vec<&'static str> = names.iter().copied().filter(|n| STRUCTURAL.contains(n)).collect();
        if structural.len() > MAX_EXPECTED || (structural == ["'}'"] && found != "yyeof") {
            return Self::from_slice(&[]);
        }
        Self::from_slice(&structural)
    }

    fn from_slice(names: &[&'static str]) -> Self {
        let mut buf = [""; MAX_EXPECTED];
        buf[..names.len()].copy_from_slice(names);
        Self {
            names: buf,
            len: names.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Display for ExpectedTokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, name) in self.names[..self.len].iter().map(|n| token_label(n)).enumerate() {
            match i {
                0 => write!(f, "{name}")?,
                i if i + 1 == self.len => write!(f, " or {name}")?,
                _ => write!(f, ", {name}")?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Diagnosis {
    DivisionByZero,
    BadArgumentsCount,
    SyntaxError {
        found: &'static str,
        expected: ExpectedTokens,
    },
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
    NonIntArraySize,
    TagWithoutMember(SymbolKind),
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
    VoidParameter,
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
            Diagnosis::DivisionByZero => Severity:: Error,
            Diagnosis::BadArgumentsCount => Severity::Error,
            Diagnosis::SyntaxError { .. } => Severity::Error,
            Diagnosis::InvalidSizeof => Severity::Error,
            Diagnosis::UndeclaredIdentifier(_) => Severity::Error,
            Diagnosis::NonConstantExpression => Severity::Error,
            Diagnosis::NonIntegerConstantExpression => Severity::Error,
            Diagnosis::CastToNonScalar => Severity::Error,
            Diagnosis::EmptyDeclaration => Severity::Error,
            Diagnosis::TagWithoutMember(_) => Severity::Error,
            Diagnosis::MultipleStorageSpecifiers => Severity::Error,
            Diagnosis::BlockScopeNotExtern => Severity::Error,
            Diagnosis::InvalidTypeSpecifer => Severity::Error,
            Diagnosis::DuplicateTypeQualifers => Severity::Error,
            Diagnosis::NonIntBitFieldType => Severity::Error,
            Diagnosis::NonIntArraySize => Severity::Error,
            Diagnosis::VariantBadValue => Severity::Error,
            Diagnosis::AutoRegisterExternal => Severity::Error,
            Diagnosis::NotFunctionTypeDeclarator => Severity::Error,
            Diagnosis::FunctionAutoExtern => Severity::Error,
            Diagnosis::ParameterOldStyleListLenMismatch => Severity::Error,
            Diagnosis::ParameterTypeListWithList => Severity::Error,
            Diagnosis::ParameterNotRegister => Severity::Error,
            Diagnosis::VoidParameter => Severity::Error,
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
            Diagnosis::DivisionByZero => "Division by zero".to_string(),
            Diagnosis::BadArgumentsCount => "wrong argument count".to_string(),
            Diagnosis::SyntaxError { found, expected } if expected.is_empty() => format!("syntax error, unexpected {}", token_label(found)),
            Diagnosis::SyntaxError { found, expected } => format!("syntax error, unexpected {}, expecting {expected}", token_label(found)),
            Diagnosis::InvalidSizeof => "invalid application of sizeof".to_string(),
            Diagnosis::UndeclaredIdentifier(name) => format!("Use of undeclared identifier '{}'", name.id.resolve(ctx)),
            Diagnosis::NonConstantExpression => "Non constant expression".to_string(),
            Diagnosis::NonIntegerConstantExpression => "Non integer constant expression".to_string(),
            Diagnosis::CastToNonScalar => "Conversion to non scalar type requested".to_string(),
            Diagnosis::VariantBadValue => "Variant value should be in int range".to_string(),
            Diagnosis::EmptyDeclaration => "Declaration declares nothing".to_string(),
            Diagnosis::TagWithoutMember(kind) => format!("{kind} has no named member"),
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
            Diagnosis::VoidParameter => "Parameter shall not have void type".to_string(),
            Diagnosis::AbstractParameterDeclaration => "Absctract declaration in old style function".to_string(),
            Diagnosis::MissingDeclarationInOldStyle => "Missing argument declaration in old style function".to_string(),
            Diagnosis::MissingParameterInOldStyle => "Missing parameter".to_string(),
            Diagnosis::DuplicateParameterName => "Duplicate paramter identifier".to_string(),
            Diagnosis::TypedefInOldStyle => "Typedef unsed in old style function".to_string(),
            Diagnosis::LabelOutsideFunction => "Label outside function".to_string(),
            Diagnosis::DuplicateDeclaration(kind, name) => format!("duplicate declaration of {} `{}'", kind, name.id.resolve(ctx)),
            Diagnosis::NonIntBitFieldType => "Bit-field has non-integral type".to_string(),
            Diagnosis::NonIntArraySize => "Array len has non-integral type".to_string(),
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

pub fn report<W: Write, D: Display>(
    w: &mut W,
    ctx: &Context,
    span: Span,
    severity: Severity,
    msg: D,
) -> io::Result<()> {
    let line_no = span.start.line;
    let padding = line_no.to_string().len();
    let mid_pad = 9 - padding;

    let path = ctx.file_of(span);
    let color = severity.color();

    writeln!(
        w,
        "{path}:{line_no}:{}: {color}{severity}:{RESET} {msg}",
        span.start.col
    )?;

    let Ok(file) = File::open(path) else { return Ok(()) };
    let Some(Ok(line)) = BufReader::new(file).lines().nth(line_no - 1) else { return Ok(()) };
    let line = line.replace('\t', " ");
    let col_no = caret_end(span, &line);

    writeln!(w, "     {:>padding$}|{:>mid_pad$}{line}", line_no, "")?;
    writeln!(
        w,
        "     {:>padding$}|{:>mid_pad$}{color}{:>col_no$}{RESET} ",
        "",
        "",
        "^".repeat(col_no + 1 - span.start.col)
    )
}

fn caret_end(span: Span, line: &str) -> usize {
    match span.start.line == span.end.line {
        true => span.end.col.max(span.start.col),
        false => line.len().max(span.start.col),
    }
}
