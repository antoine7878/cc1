use std::fmt::{self, Display};
use std::io::{self, Write, stderr};

use crate::ast::{Name, StringId, UnaryOp};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{QualifiedType, SymbolKind};
use crate::utils::{RED, RESET, YELLOW};

#[derive(Clone, Debug)]
pub enum Diagnosis {
    Temprorary,
    InvalidReturnType,
    ArrayInitTooLong,
    Poisoned,
    InvalidOperand,
    ModuloByZero,
    DivisionByZero,
    BadArgumentsCount,
    SyntaxError {
        found: &'static str,
        expected: ExpectedTokens,
    },
    InvalidSizeof,
    UndeclaredIdentifier(Name),
    /// 6.1.4
    MixedWideStringConcat,
    /// 6.1.3.2
    IntegerConstantTooLarge,
    /// 6.4
    ConstantOverflow,
    /// 6.3
    ArithmeticOverflow,
    /// 6.3.2.1
    SubscriptNotArray,
    /// 6.3.2.2
    TooManyArguments(usize, usize),
    TooFewArguments(usize, usize),
    /// 6.3.2.3
    AccessNotStuctOrUnion(QualifiedType),
    AccessNotPointer(QualifiedType),
    AccessNotMember(QualifiedType, StringId),
    /// 6.3.7
    ShiftCountNegative,
    ShiftCountOutOfRange,
    /// 6.2.2.1
    IncompleteType(QualifiedType),

    /// 6.3.2.4
    BadPostIncDec(UnaryOp, QualifiedType),
    InvalidUnary(QualifiedType),
    /// 6.3.4
    CastToNonScalar,
    CastOfNonScalar,
    IncompatibleCast,
    /// 6.3.16
    AssignmentDiscardedQualifiers(QualifiedType, QualifiedType),
    InitDiscardedQualifiers(QualifiedType, QualifiedType),
    ArgumentDiscardedQualifiers(usize, QualifiedType, QualifiedType),
    ReturnDiscardedQualifiers(QualifiedType, QualifiedType),

    AssignmentIncompatibleTypes(QualifiedType, QualifiedType),
    InitIncompatibleTypes(QualifiedType, QualifiedType),
    ArgumentIncompatibleTypes(usize, QualifiedType, QualifiedType),
    ReturnIncompatibleTypes(QualifiedType, QualifiedType),

    CallingNotFunction(QualifiedType),
    CallingIncompleteReturn(QualifiedType),

    /// 6.3.5
    InvalidBianryOperand(QualifiedType, QualifiedType),

    AssignToRValue,
    ConstAssignment(QualifiedType),
    // InvalidCast,
    /// 6.4
    NonConstantExpression,
    NonIntegerConstantExpression,
    /// 6.5
    EmptyDeclaration,
    MultipleStorageSpecifiers,
    BlockScopeNotExtern,
    InvalidTypeSpecifier,
    DuplicateTypeQualifiers,
    /// 6.5.4.3
    FunctionReturningArray(QualifiedType),
    FunctionReturningFunction(QualifiedType),
    /// 6.5.2.1
    NonIntBitFieldType,
    NonIntArraySize,
    TagWithoutMember(SymbolKind),
    /// 6.5.2.2
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
    UnnamedPrototypeParameter,
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
            Diagnosis::MixedWideStringConcat => Severity::Warning,
            _ => Severity::Error
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
        let sema = &ctx.sema;
        match &self.inner {
            Diagnosis::SubscriptNotArray => "subscripted value is not an array, pointer, or vector".to_string(),

            Diagnosis::TooManyArguments(expected, have) => format!("too many arguments to function call, expected {expected}, have {have}"),
            Diagnosis::TooFewArguments(expected, have) => format!("too few arguments to function call, expected {expected}, have {have}"),

            Diagnosis::AccessNotStuctOrUnion(ty) => format!("member reference base type '{}' is not a structure or union", ty.describe(sema, ctx)),
            Diagnosis::AccessNotPointer(ty) => format!("member reference base type '{}' is not pointer", ty.describe(sema, ctx)),
            Diagnosis::AccessNotMember(ty, name_id ) => format!("no member named '{}' in '{}'", name_id.resolve(ctx), ty.describe(sema, ctx)),

            Diagnosis::Temprorary =>  "TEMPRORARY DIAG".to_string(),
            Diagnosis::ArrayInitTooLong => "excess elements in array initializer".to_string(), Diagnosis::InvalidReturnType => "Invalid return type".to_string(),

            Diagnosis::AssignmentDiscardedQualifiers(to, from) => format!("assigning to ‘{}’ from ‘{}’ discards qualifiers", to.describe(&ctx.sema, ctx), from.describe(sema, ctx)),
            Diagnosis::InitDiscardedQualifiers(to, from) => format!("initializing ‘{}’ with an expression of type ‘{}’ discards qualifiers", to.describe(sema, ctx), from.describe(sema, ctx)),
            Diagnosis::ArgumentDiscardedQualifiers(n, to, from) => format!("passing ‘{}’ to parameter {n} of type ‘{}’ discards qualifiers", from.describe(sema, ctx), to.describe(sema, ctx)),
            Diagnosis::ReturnDiscardedQualifiers(to, from) => format!("returning ‘{}’ from a function with result type ‘{}’ discards qualifiers", from.describe(sema, ctx), to.describe(sema, ctx)),

            Diagnosis::AssignmentIncompatibleTypes(to, from) => format!("assignment to ‘{}’ from incompatible pointer type ‘{}’", to.describe(sema, ctx), from.describe(sema, ctx)),
            Diagnosis::InitIncompatibleTypes(to, from) => format!("initialization of ‘{}’ from incompatible pointer type ‘{}’", to.describe(sema, ctx), from.describe(sema, ctx)),
            Diagnosis::ArgumentIncompatibleTypes(n, to, from) => format!("passing ‘{}’ to parameter {n} of incompatible type ‘{}’", from.describe(sema, ctx), to.describe(sema, ctx)),
            Diagnosis::ReturnIncompatibleTypes(to, from) => format!("returning ‘{}’ from a function with incompatible result type ‘{}’", from.describe(sema, ctx), to.describe(sema, ctx)),

            Diagnosis::CallingNotFunction(ty) => format!("called object type '{}' is not a function or function pointer", ty.describe(sema, ctx)),
            Diagnosis::CallingIncompleteReturn(ty) => format!("calling a function with incomplete return type ‘{}’", ty.describe(sema, ctx)),
            Diagnosis::BadPostIncDec(UnaryOp::PostInc, ty) => format!("cannot increment value of type '{}'", ty.describe(sema, ctx)),
            Diagnosis::BadPostIncDec(UnaryOp::PostDec, ty) => format!("cannot decrement value of type '{}'", ty.describe(sema, ctx)),
            Diagnosis::BadPostIncDec(_, _) => unreachable!(),

            Diagnosis::InvalidUnary(ty) => format!("invalid argument type '{}' to unary expression", ty.describe(sema, ctx)),

            Diagnosis::FunctionReturningArray(ty) => format!("function cannot return array type ‘{}’", ty.describe(sema, ctx)),
            Diagnosis::FunctionReturningFunction(ty) => format!("function cannot return function type ‘{}’", ty.describe(sema, ctx)),
            Diagnosis::InvalidBianryOperand(lhs, rhs) => format!("invalid operands to binary expression ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),
            Diagnosis::AssignToRValue => "expression is not assignable".to_string(),
            Diagnosis::ConstAssignment(ty) => format!("cannot assign to variable with const-qualified type '{}'", ty.describe(sema, ctx)),
            Diagnosis::Poisoned => "Internal error".to_string(),
            Diagnosis::InvalidOperand => "invalid operand".to_string(),
            Diagnosis::IncompleteType(ty) => format!("incomplete definition of type '{}'", ty.describe(sema, ctx)),
            Diagnosis::ModuloByZero =>  "remainder by zero is undefined".to_string(),
            Diagnosis::DivisionByZero => "division by zero is undefined".to_string(),
            Diagnosis::ConstantOverflow => "overflow in constant expression".to_string(),
            Diagnosis::ArithmeticOverflow => "integer overflow in constant expression".to_string(),
            Diagnosis::ShiftCountNegative => "shift count is negative".to_string(),
            Diagnosis::ShiftCountOutOfRange => "shift count >= width of type".to_string(),
            Diagnosis::BadArgumentsCount => "wrong argument count".to_string(),
            Diagnosis::SyntaxError { found, expected } if expected.is_empty() => format!("syntax error, unexpected {}", token_label(found)),
            Diagnosis::SyntaxError { found, expected } => format!("syntax error, unexpected {}, expecting {expected}", token_label(found)),
            Diagnosis::InvalidSizeof => "invalid application of sizeof".to_string(),
            Diagnosis::UndeclaredIdentifier(name) => format!("Use of undeclared identifier '{}'", name.id.resolve(ctx)),
            Diagnosis::IntegerConstantTooLarge => "integer constant is too large for any integer type".to_string(),
            Diagnosis::MixedWideStringConcat => "concatenation of a wide and a narrow string literal is undefined".to_string(),
            Diagnosis::NonConstantExpression => "Non constant expression".to_string(),
            Diagnosis::NonIntegerConstantExpression => "Non integer constant expression".to_string(),
            Diagnosis::CastToNonScalar => "Conversion to non scalar type".to_string(),
            Diagnosis::CastOfNonScalar => "Conversion of non scalar type".to_string(),
            Diagnosis::IncompatibleCast => "Incompatible types".to_string(),
            Diagnosis::VariantBadValue => "Variant value should be in int range".to_string(),
            Diagnosis::EmptyDeclaration => "Declaration declares nothing".to_string(),
            Diagnosis::TagWithoutMember(kind) => format!("{kind} has no named member"),
            Diagnosis::MultipleStorageSpecifiers => "Multiple storage class declaration".to_string(),
            Diagnosis::BlockScopeNotExtern => "Function in block not declared as extern".to_string(),
            Diagnosis::InvalidTypeSpecifier => "Invalid type specifier or combination thereof".to_string(),
            Diagnosis::DuplicateTypeQualifiers => "Duplicate type qualifiers".to_string(),
            Diagnosis::AutoRegisterExternal => "External declaration auto of register".to_string(),
            Diagnosis::NotFunctionTypeDeclarator => "Declarator shall be function type".to_string(),
            Diagnosis::FunctionAutoExtern => "Function storage shall be auto or extern".to_string(),
            Diagnosis::ParameterOldStyleListLenMismatch => "Old style parameter function declaration shall be followed by a declaration list".to_string(),
            Diagnosis::ParameterTypeListWithList => "Parameter style function declration shall not be followed by a declaration list".to_string(),
            Diagnosis::ParameterNotRegister => "Parameter shall only by declared with register storage".to_string(),
            Diagnosis::VoidParameter => "Parameter shall not have void type".to_string(),
            Diagnosis::UnnamedPrototypeParameter => "Parameter shall include an identifier".to_string(),
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
pub trait DiagCollector {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode>;

    fn add_diag<T>(&mut self, diag: Diag<T>, span: &Span) -> T {
        match diag.diagnosis {
            Some(Diagnosis::Poisoned) | None => (),
            Some(diagnosis) => self.diagnosis().push(DiagnosisNode::new(diagnosis, *span)),
        }
        diag.res
    }
}

#[derive(Debug)]
pub struct Diag<T> {
    pub res: T,
    pub diagnosis: Option<Diagnosis>,
}

impl<T> Diag<T> {
    pub fn collect<C: DiagCollector>(self, collector: &mut C, span: &Span) -> T {
        collector.add_diag(self, span)
    }

    pub fn new(res: T, diagnosis: Option<Diagnosis>) -> Self {
        Self { res, diagnosis }
    }

    pub fn ok(res: T) -> Self {
        Self::new(res, None)
    }

    pub fn err(res: T, diagnosis: Diagnosis) -> Self {
        Self::new(res, Some(diagnosis))
    }
}

#[derive(Debug, Clone)]
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
    let mid_pad = 9usize.saturating_sub(padding);

    let path = ctx.file_of(span);
    let color = severity.color();

    writeln!(
        w,
        "{path}:{line_no}:{}: {color}{severity}:{RESET} {msg}",
        span.start.col,
    )?;

    let Some(line) = ctx.source_line(path, line_no) else { return Ok(()) };
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
