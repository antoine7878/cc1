use std::fmt::{self, Display};
use std::io::{self, Write, stderr};

use crate::ast::{Name, StringId, UnaryOp, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{QualifiedType, SymbolKind};
use crate::utils::{RED, RESET, YELLOW};

#[derive(Clone, Debug)]
pub enum Diagnosis {
    OutsideSwitch(&'static str),
    DuplicateCase(Value),
    DuplicateDefault,
    BreakNotInLoop,
    ContinueNotInLoop,
    DuplicateLabel(Name),
    UndefinedLabel(Name),

    Poisoned,
    InvalidOperand,
    SyntaxError {
        found: &'static str,
        expected: ExpectedTokens,
    },

    // 6.1.2.1
    DuplicateDeclaration(SymbolKind, Name),

    // 6.1.2.2
    ConflictingLinkage(Name),

    // 6.1.3.2
    IntegerConstantTooLarge,

    // 6.1.3.4
    EscapeOutOfRange,

    // 6.1.4
    MixedWideStringConcat,

    // 6.2.2.1
    IncompleteType(QualifiedType),

    // 6.3
    ArithmeticOverflow,

    // 6.3.1
    UndeclaredIdentifier(Name),

    // 6.3.2.1
    SubscriptNotArray,

    // 6.3.2.2
    CallingNotFunction(QualifiedType),
    CallingIncompleteReturn(QualifiedType),
    TooManyArguments(usize, usize),
    TooFewArguments(usize, usize),
    BadArgumentsCount,
    ArgumentDiscardedQualifiers(usize, QualifiedType, QualifiedType),
    ArgumentIncompatibleTypes(usize, QualifiedType, QualifiedType),

    // 6.3.2.3
    AccessNotStuctOrUnion(QualifiedType),
    AccessNotPointer(QualifiedType),
    AccessNotMember(QualifiedType, StringId),

    // 6.3.2.4
    BadPostIncDec(UnaryOp, QualifiedType),

    // 6.3.3.2
    BitFieldAddress,
    RegisterAddress,
    RValueAddress(QualifiedType),
    IndirectionNotPointer(QualifiedType),
    IndirectionToVoid,

    // 6.3.3.3
    InvalidUnary(QualifiedType),

    // 6.3.3.4
    SizeofVoid,
    SizeofIncomplete(QualifiedType),
    SizeofFunction,
    SizeofBitfield,

    // 6.3.4
    CastToNonScalar,
    CastOfNonScalar,

    // 6.3.5
    DivisionByZero,
    ModuloByZero,
    InvalidBinaryOperand(QualifiedType, QualifiedType),

    // 6.3.7
    ShiftCountNegative,
    ShiftCountOutOfRange,

    // 6.3.8
    OrderedFunctionPointers(QualifiedType, QualifiedType),
    MixedCompletenessComparison(QualifiedType, QualifiedType),

    // 6.3.9
    InvalidComparison(QualifiedType, QualifiedType),

    // 6.3.15
    NotScalar(QualifiedType),
    IncompatibleOperands(QualifiedType, QualifiedType),
    PointerMismatch(QualifiedType, QualifiedType),

    // 6.3.16
    AssignToRValue,
    ConstAssignment(QualifiedType),
    ConstMemberAssignment(QualifiedType),
    AssignmentDiscardedQualifiers(QualifiedType, QualifiedType),
    AssignmentIncompatibleTypes(QualifiedType, QualifiedType),

    // 6.4
    ConstantOverflow,
    NonConstantExpression,
    NonIntegerConstantExpression,

    // 6.5
    EmptyDeclaration,
    InvalidTypeSpecifier,
    IncompleteVariable(QualifiedType),

    // 6.5.1
    MultipleStorageSpecifiers,
    BlockScopeNotExtern,

    // 6.5.2.1
    TagWithoutMember(SymbolKind),
    InvalidMemberType(QualifiedType),
    NonIntBitFieldType,
    NegativeBitFieldWidth(Option<Name>, i64),
    BitFieldWidthTooLarge(Option<Name>, u64, u32),
    ZeroWidthNamedBitField(Name),

    // 6.5.2.2
    VariantBadValue,

    // 6.5.2.3
    ForwardEnumReference(Option<Name>),

    // 6.5.3
    DuplicateTypeQualifiers,

    // 6.5.4.2
    NonIntArraySize,
    NegativeArraySize,
    ZeroArraySize,
    InvalidElementType(QualifiedType),

    // 6.5.4.3
    FunctionReturningArray(QualifiedType),
    FunctionReturningFunction(QualifiedType),
    VoidParameter,
    ParameterNotRegister,
    DuplicateParameterName,

    // 6.5.7
    BlockScopeLinkageInitializer,
    ArrayInitTooLong,
    NonConstantInitializer,
    InitDiscardedQualifiers(QualifiedType, QualifiedType),
    InitIncompatibleTypes(QualifiedType, QualifiedType),

    // 6.6.1

    // 6.6.4
    NonScalarStatement(QualifiedType),

    // 6.6.4.2
    NonIntegralStatement(QualifiedType),

    // 6.6.6.4
    ReturnWithoutValue,
    ReturnDiscardedQualifiers(QualifiedType, QualifiedType),
    ReturnIncompatibleTypes(QualifiedType, QualifiedType),

    // 6.7
    AutoRegisterExternal,
    InternalNeverDefined(Name),
    TentativeNeverCompleted(QualifiedType),

    // 6.7.1
    NotFunctionTypeDeclarator,
    FunctionAutoExtern,
    UnnamedPrototypeParameter,
    ParameterTypeListWithList,
    MissingParameterInOldStyle,
    IncompleteParameter(QualifiedType),
    IncompleteReturn(QualifiedType),
}

impl Diagnosis {
    #[rustfmt::skip]
    pub fn severity(&self) -> Severity {
        match self {
            Diagnosis::MixedWideStringConcat
            | Diagnosis::ReturnWithoutValue
            | Diagnosis::ShiftCountNegative
            | Diagnosis::ShiftCountOutOfRange => Severity::Warning,
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
            Diagnosis::OutsideSwitch(s) =>format!("'{}' statement not in switch statement", s),
            Diagnosis::DuplicateCase(value) => format!("duplicate case value '{}'", value.to_u64()),
            Diagnosis::DuplicateDefault => "multiple default labels in one switch".to_string(),
            Diagnosis::BreakNotInLoop => "'break' statement not in loop or switch statement".to_string(),
            Diagnosis::ContinueNotInLoop => "'continue' statement not in loop statement".to_string(),
            Diagnosis::DuplicateLabel(name) => format!("redefinition of label '{}'", name.id.resolve(ctx)),
            Diagnosis::UndefinedLabel(name) => format!("use of undeclared label '{}'", name.id.resolve(ctx)),

            Diagnosis::Poisoned => "Internal error".to_string(),
            Diagnosis::InvalidOperand => "invalid operand".to_string(),
            Diagnosis::SyntaxError { found, expected } if expected.is_empty() => format!("syntax error, unexpected {}", token_label(found)),
            Diagnosis::SyntaxError { found, expected } => format!("syntax error, unexpected {}, expecting {expected}", token_label(found)),

            // 6.1.2.1
            Diagnosis::DuplicateDeclaration(kind, name) => format!("duplicate declaration of {} `{}'", kind, name.id.resolve(ctx)),

            // 6.1.2.2
            Diagnosis::ConflictingLinkage(name) => format!("declaration of '{}' conflicts with the linkage of a previous declaration", name.id.resolve(ctx)),

            // 6.1.3.2
            Diagnosis::IntegerConstantTooLarge => "integer constant is too large for any integer type".to_string(),

            // 6.1.3.4
            Diagnosis::EscapeOutOfRange => "escape sequence is out of range for the character type".to_string(),

            // 6.1.4
            Diagnosis::MixedWideStringConcat => "concatenation of a wide and a narrow string literal is undefined".to_string(),

            // 6.2.2.1
            Diagnosis::IncompleteType(ty) => format!("incomplete definition of type '{}'", ty.describe(sema, ctx)),

            // 6.3
            Diagnosis::ArithmeticOverflow => "integer overflow in constant expression".to_string(),

            // 6.3.1
            Diagnosis::UndeclaredIdentifier(name) => format!("Use of undeclared identifier '{}'", name.id.resolve(ctx)),

            // 6.3.2.1
            Diagnosis::SubscriptNotArray => "subscripted value is not an array, pointer, or vector".to_string(),

            // 6.3.2.2
            Diagnosis::CallingNotFunction(ty) => format!("called object type '{}' is not a function or function pointer", ty.describe(sema, ctx)),
            Diagnosis::CallingIncompleteReturn(ty) => format!("calling a function with incomplete return type '{}'", ty.describe(sema, ctx)),
            Diagnosis::TooManyArguments(expected, have) => format!("too many arguments to function call, expected {expected}, have {have}"),
            Diagnosis::TooFewArguments(expected, have) => format!("too few arguments to function call, expected {expected}, have {have}"),
            Diagnosis::BadArgumentsCount => "wrong argument count".to_string(),
            Diagnosis::ArgumentDiscardedQualifiers(n, to, from) => format!("passing '{}' to parameter {n} of type '{}' discards qualifiers", from.describe(sema, ctx), to.describe(sema, ctx)),
            Diagnosis::ArgumentIncompatibleTypes(n, to, from) => format!("passing '{}' to parameter {n} of incompatible type '{}'", from.describe(sema, ctx), to.describe(sema, ctx)),

            // 6.3.2.3
            Diagnosis::AccessNotStuctOrUnion(ty) => format!("member reference base type '{}' is not a structure or union", ty.describe(sema, ctx)),
            Diagnosis::AccessNotPointer(ty) => format!("member reference base type '{}' is not pointer", ty.describe(sema, ctx)),
            Diagnosis::AccessNotMember(ty, name_id ) => format!("no member named '{}' in '{}'", name_id.resolve(ctx), ty.describe(sema, ctx)),

            // 6.3.2.4
            Diagnosis::BadPostIncDec(UnaryOp::PostInc | UnaryOp::PreInc, ty) => format!("cannot increment value of type '{}'", ty.describe(sema, ctx)),
            Diagnosis::BadPostIncDec(UnaryOp::PostDec | UnaryOp::PreDec, ty) => format!("cannot decrement value of type '{}'", ty.describe(sema, ctx)),
            Diagnosis::BadPostIncDec(_, _) => unreachable!(),

            // 6.3.3.2
            Diagnosis::BitFieldAddress => "address of bit-field requested".to_string(),
            Diagnosis::RegisterAddress => "address of register variable requested".to_string(),
            Diagnosis::RValueAddress(ty) => format!( "cannot take the address of an rvalue of type '{}'", ty.describe(sema, ctx)),
            Diagnosis::IndirectionNotPointer(ty) => format!("indirection requires pointer operand ('{}' invalid)", ty.describe(sema, ctx)),
            Diagnosis::IndirectionToVoid => "ISO C does not allow indirection on operand of type 'void *'".to_string(),

            // 6.3.3.3
            Diagnosis::InvalidUnary(ty) => format!("invalid argument type '{}' to unary expression", ty.describe(sema, ctx)),

            // 6.3.3.4
            Diagnosis::SizeofVoid => "invalid application of 'sizeof' to a void type".to_string(),
            Diagnosis::SizeofIncomplete(ty) => format!("invalid application of 'sizeof' to an incomplete type '{}'", ty.describe(sema, ctx)),
            Diagnosis::SizeofFunction => "invalid application of 'sizeof' to a function type".to_string(),
            Diagnosis::SizeofBitfield => "invalid application of 'sizeof' to bit-field".to_string(),

            // 6.3.4
            Diagnosis::CastToNonScalar => "Conversion to non scalar type".to_string(),
            Diagnosis::CastOfNonScalar => "Conversion of non scalar type".to_string(),

            // 6.3.5
            Diagnosis::DivisionByZero => "division by zero is undefined".to_string(),
            Diagnosis::ModuloByZero =>  "remainder by zero is undefined".to_string(),
            Diagnosis::InvalidBinaryOperand(lhs, rhs) => format!("invalid operands to binary expression ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),

            // 6.3.7
            Diagnosis::ShiftCountNegative => "shift count is negative".to_string(),
            Diagnosis::ShiftCountOutOfRange => "shift count >= width of type".to_string(),

            // 6.3.8
            Diagnosis::OrderedFunctionPointers(lhs, rhs) => format!("ordered comparison of function pointers ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),
            Diagnosis::MixedCompletenessComparison(lhs, rhs) => format!("ordered comparison needs two complete or two incomplete pointee types ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),

            // 6.3.9
            Diagnosis::InvalidComparison(lhs, rhs) => format!("comparison of distinct pointer types ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),

            // 6.3.15
            Diagnosis::NotScalar(ty) => format!("used type '{}' where arithmetic or pointer type is required", ty.describe(sema, ctx)),
            Diagnosis::IncompatibleOperands(lhs, rhs) => format!("incompatible operand types ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),
            Diagnosis::PointerMismatch(lhs, rhs) => format!("pointer type mismatch ('{}' and '{}')", lhs.describe(sema, ctx), rhs.describe(sema, ctx)),

            // 6.3.16
            Diagnosis::AssignToRValue => "expression is not assignable".to_string(),
            Diagnosis::ConstAssignment(ty) => format!("cannot assign to variable with const-qualified type '{}'", ty.describe(sema, ctx)),
            Diagnosis::ConstMemberAssignment(ty) => format!("cannot assign to '{}' because it has a const-qualified member", ty.describe(sema, ctx)),
            Diagnosis::AssignmentDiscardedQualifiers(to, from) => format!("assigning to '{}' from '{}' discards qualifiers", to.describe(&ctx.sema, ctx), from.describe(sema, ctx)),
            Diagnosis::AssignmentIncompatibleTypes(to, from) => format!("assignment to '{}' from incompatible pointer type '{}'", to.describe(sema, ctx), from.describe(sema, ctx)),

            // 6.4
            Diagnosis::ConstantOverflow => "overflow in constant expression".to_string(),
            Diagnosis::NonConstantExpression => "Non constant expression".to_string(),
            Diagnosis::NonIntegerConstantExpression => "Non integer constant expression".to_string(),

            // 6.5
            Diagnosis::EmptyDeclaration => "Declaration declares nothing".to_string(),
            Diagnosis::InvalidTypeSpecifier => "Invalid type specifier or combination thereof".to_string(),
            Diagnosis::IncompleteVariable(ty) => format!("variable has incomplete type '{}'", ty.describe(sema, ctx)),

            // 6.5.1
            Diagnosis::MultipleStorageSpecifiers => "Multiple storage class declaration".to_string(),
            Diagnosis::BlockScopeNotExtern => "Function in block not declared as extern".to_string(),

            // 6.5.2.1
            Diagnosis::TagWithoutMember(kind) => format!("{kind} has no named member"),
            Diagnosis::InvalidMemberType(ty) => format!("field has incomplete or function type '{}'", ty.describe(sema, ctx)),
            Diagnosis::NonIntBitFieldType => "Bit-field has non-integral type".to_string(),
            Diagnosis::NegativeBitFieldWidth(Some(name), width) => format!("bit-field '{}' has negative width ({width})", name.id.resolve(ctx)),
            Diagnosis::NegativeBitFieldWidth(None, width) => format!("anonymous bit-field has negative width ({width})"),
            Diagnosis::BitFieldWidthTooLarge(Some(name), width, bits) => format!("width of bit-field '{}' ({width} bits) exceeds the width of its type ({bits} bits)", name.id.resolve(ctx)),
            Diagnosis::BitFieldWidthTooLarge(None, width, bits) => format!("width of anonymous bit-field ({width} bits) exceeds the width of its type ({bits} bits)"),
            Diagnosis::ZeroWidthNamedBitField(name) => format!("named bit-field '{}' has zero width", name.id.resolve(ctx)),

            // 6.5.2.2
            Diagnosis::ForwardEnumReference(Some(name)) => format!("ISO C forbids forward references to enum '{}'", name.id.resolve(ctx)),
            Diagnosis::ForwardEnumReference(None) => "ISO C forbids forward references to 'enum' types".to_string(),

            Diagnosis::VariantBadValue => "Variant value should be in int range".to_string(),

            // 6.5.3
            Diagnosis::DuplicateTypeQualifiers => "Duplicate type qualifiers".to_string(),

            // 6.5.4.2
            Diagnosis::NonIntArraySize => "Array len has non-integral type".to_string(),
            Diagnosis::NegativeArraySize => "size of array is negative".to_string(),
            Diagnosis::ZeroArraySize => "size of array is zero".to_string(),
            Diagnosis::InvalidElementType(ty) => format!("array has incomplete or function element type '{}'", ty.describe(sema, ctx)),

            // 6.5.4.3
            Diagnosis::FunctionReturningArray(ty) => format!("function cannot return array type '{}'", ty.describe(sema, ctx)),
            Diagnosis::FunctionReturningFunction(ty) => format!("function cannot return function type '{}'", ty.describe(sema, ctx)),
            Diagnosis::VoidParameter => "Parameter shall not have void type".to_string(),
            Diagnosis::ParameterNotRegister => "Parameter shall only by declared with register storage".to_string(),
            Diagnosis::DuplicateParameterName => "Duplicate paramter identifier".to_string(),

            // 6.5.7
            Diagnosis::ArrayInitTooLong => "excess elements in array initializer".to_string(),
            Diagnosis::BlockScopeLinkageInitializer => "declaration of block scope identifier with linkage cannot have an initializer".to_string(),
            Diagnosis::NonConstantInitializer => "initializer element is not a compile-time constant".to_string(),
            Diagnosis::InitDiscardedQualifiers(to, from) => format!("initializing '{}' with an expression of type '{}' discards qualifiers", to.describe(sema, ctx), from.describe(sema, ctx)),
            Diagnosis::InitIncompatibleTypes(to, from) => format!("initialization of '{}' from incompatible pointer type '{}'", to.describe(sema, ctx), from.describe(sema, ctx)),

            // 6.6.1

            // 6.6.4
            Diagnosis::NonScalarStatement(ty) => format!("statement requires expression of scalar type ('{}' invalid)", ty.describe(sema, ctx)),

            // 6.6.4.2
            Diagnosis::NonIntegralStatement(ty) => format!("statement requires expression of integer type ('{}' invalid)", ty.describe(sema, ctx)),

            // 6.6.6.4
            Diagnosis::ReturnWithoutValue => "'return' with no value, in function returning non-void".to_string(),
            Diagnosis::ReturnDiscardedQualifiers(to, from) => format!("returning '{}' from a function with result type '{}' discards qualifiers", from.describe(sema, ctx), to.describe(sema, ctx)),
            Diagnosis::ReturnIncompatibleTypes(to, from) => format!("returning '{}' from a function with incompatible result type '{}'", from.describe(sema, ctx), to.describe(sema, ctx)),

            // 6.7
            Diagnosis::AutoRegisterExternal => "External declaration auto of register".to_string(),
            Diagnosis::InternalNeverDefined(name) => format!("'{}' used but never defined", name.id.resolve(ctx)),
            Diagnosis::TentativeNeverCompleted(ty) => format!("tentative definition has type '{}' that is never completed", ty.describe(sema, ctx)),

            // 6.7.1
            Diagnosis::NotFunctionTypeDeclarator => "Declarator shall be function type".to_string(),
            Diagnosis::FunctionAutoExtern => "Function storage shall be auto or extern".to_string(),
            Diagnosis::UnnamedPrototypeParameter => "Parameter shall include an identifier".to_string(),
            Diagnosis::ParameterTypeListWithList => "Parameter style function declration shall not be followed by a declaration list".to_string(),
            Diagnosis::MissingParameterInOldStyle => "Missing parameter".to_string(),
            Diagnosis::IncompleteParameter(ty) => format!("parameter has incomplete type '{}'", ty.describe(sema, ctx)),
            Diagnosis::IncompleteReturn(ty) => format!("function definition has incomplete return type '{}'", ty.describe(sema, ctx)),
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

    pub fn into_result(self) -> Result<T, Diagnosis> {
        match self.diagnosis {
            Some(diagnosis) => Err(diagnosis),
            None => Ok(self.res),
        }
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
