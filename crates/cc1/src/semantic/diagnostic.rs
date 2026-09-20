use std::fmt::{self, Display};
use std::io::{self, Write, stderr};

use libft::{Severity, Span, render};

use crate::ast::{ConstValue, Name, NameId, UnaryOp};
use crate::context::ctx;
use crate::semantic::{QualifiedType, Sema, SymbolKind, sema};

#[derive(Clone, Debug)]
pub enum Diagnostic {
    OutsideSwitch(&'static str),
    DuplicateCase(ConstValue),
    DuplicateDefault,
    BreakNotInLoop,
    ContinueNotInLoop,
    DuplicateLabel(Name),
    UndefinedLabel(Name),

    BadArguments(String),
    Poisoned,
    Invariant(&'static str),
    InvalidOperand,
    SyntaxError { found: &'static str, expected: ExpectedTokens },

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
    ArgumentDiscardedQualifiers(usize, QualifiedType, QualifiedType),
    ArgumentIncompatibleTypes(usize, QualifiedType, QualifiedType),

    // 6.3.2.3
    AccessNotStuctOrUnion(QualifiedType),
    AccessNotPointer(QualifiedType),
    AccessNotMember(QualifiedType, NameId),

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
    EnumeratorBadValue,

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

impl Diagnostic {
    #[rustfmt::skip]
    pub fn severity(&self) -> Severity {
        match self {
            Diagnostic::MixedWideStringConcat
            | Diagnostic::ReturnWithoutValue
            | Diagnostic::ShiftCountNegative
            | Diagnostic::ShiftCountOutOfRange => Severity::Warning,
            _ => Severity::Error
        }
    }
}

impl DiagnosticNode {
    pub fn new(inner: Diagnostic, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn severity(&self) -> Severity {
        self.inner.severity()
    }

    pub fn is_error(&self) -> bool {
        self.severity() == Severity::Error
    }

    pub fn print(&self) -> io::Result<()> {
        self.write(&mut stderr())
    }

    pub fn write<W: Write>(&self, w: &mut W) -> io::Result<()> {
        render(w, ctx(), "cc1", self.span, self.severity(), self.message(sema()))
    }

    #[rustfmt::skip]
    pub fn message(&self, sema: &Sema) -> String {
        match &self.inner {
            Diagnostic::OutsideSwitch(s) =>format!("'{}' statement not in switch statement", s),
            Diagnostic::DuplicateCase(value) => format!("duplicate case value '{}'", value.to_u64()),
            Diagnostic::DuplicateDefault => "multiple default labels in one switch".to_string(),
            Diagnostic::BreakNotInLoop => "'break' statement not in loop or switch statement".to_string(),
            Diagnostic::ContinueNotInLoop => "'continue' statement not in loop statement".to_string(),
            Diagnostic::DuplicateLabel(name) => format!("redefinition of label '{}'", name.id.resolve()),
            Diagnostic::UndefinedLabel(name) => format!("use of undeclared label '{}'", name.id.resolve()),

            Diagnostic::Poisoned => "Internal error".to_string(),
            Diagnostic::Invariant(what) => format!("internal error: {what}: violated semantic constraint"),
            Diagnostic::InvalidOperand => "invalid operand".to_string(),
            Diagnostic::SyntaxError { found, expected } if expected.is_empty() => format!("syntax error, unexpected {}", token_label(found)),
            Diagnostic::SyntaxError { found, expected } => format!("syntax error, unexpected {}, expecting {expected}", token_label(found)),

            // 6.1.2.1
            Diagnostic::DuplicateDeclaration(kind, name) => format!("duplicate declaration of {} `{}'", kind, name.id.resolve()),

            // 6.1.2.2
            Diagnostic::ConflictingLinkage(name) => format!("declaration of '{}' conflicts with the linkage of a previous declaration", name.id.resolve()),

            // 6.1.3.2
            Diagnostic::IntegerConstantTooLarge => "integer constant is too large for any integer type".to_string(),

            // 6.1.3.4
            Diagnostic::EscapeOutOfRange => "escape sequence is out of range for the character type".to_string(),

            // 6.1.4
            Diagnostic::MixedWideStringConcat => "concatenation of a wide and a narrow string literal is undefined".to_string(),

            // 6.2.2.1
            Diagnostic::IncompleteType(ty) => format!("incomplete definition of type '{}'", ty.display(sema)),

            // 6.3
            Diagnostic::ArithmeticOverflow => "integer overflow in constant expression".to_string(),

            // 6.3.1
            Diagnostic::UndeclaredIdentifier(name) => format!("Use of undeclared identifier '{}'", name.id.resolve()),

            // 6.3.2.1
            Diagnostic::SubscriptNotArray => "subscripted value is not an array, pointer, or vector".to_string(),

            // 6.3.2.2
            Diagnostic::CallingNotFunction(ty) => format!("called object type '{}' is not a function or function pointer", ty.display(sema)),
            Diagnostic::CallingIncompleteReturn(ty) => format!("calling a function with incomplete return type '{}'", ty.display(sema)),
            Diagnostic::TooManyArguments(expected, have) => format!("too many arguments to function call, expected {expected}, have {have}"),
            Diagnostic::TooFewArguments(expected, have) => format!("too few arguments to function call, expected {expected}, have {have}"),
            Diagnostic::BadArguments(error) => error.clone(),
            Diagnostic::ArgumentDiscardedQualifiers(n, to, from) => format!("passing '{}' to parameter {n} of type '{}' discards qualifiers", from.display(sema), to.display(sema)),
            Diagnostic::ArgumentIncompatibleTypes(n, to, from) => format!("passing '{}' to parameter {n} of incompatible type '{}'", from.display(sema), to.display(sema)),

            // 6.3.2.3
            Diagnostic::AccessNotStuctOrUnion(ty) => format!("member reference base type '{}' is not a structure or union", ty.display(sema)),
            Diagnostic::AccessNotPointer(ty) => format!("member reference base type '{}' is not pointer", ty.display(sema)),
            Diagnostic::AccessNotMember(ty, name_id ) => format!("no member named '{}' in '{}'", name_id.resolve(), ty.display(sema)),

            // 6.3.2.4
            Diagnostic::BadPostIncDec(UnaryOp::PostInc | UnaryOp::PreInc, ty) => format!("cannot increment value of type '{}'", ty.display(sema)),
            Diagnostic::BadPostIncDec(UnaryOp::PostDec | UnaryOp::PreDec, ty) => format!("cannot decrement value of type '{}'", ty.display(sema)),
            Diagnostic::BadPostIncDec(_, _) => unreachable!(),

            // 6.3.3.2
            Diagnostic::BitFieldAddress => "address of bit-field requested".to_string(),
            Diagnostic::RegisterAddress => "address of register variable requested".to_string(),
            Diagnostic::RValueAddress(ty) => format!( "cannot take the address of an rvalue of type '{}'", ty.display(sema)),
            Diagnostic::IndirectionNotPointer(ty) => format!("indirection requires pointer operand ('{}' invalid)", ty.display(sema)),
            Diagnostic::IndirectionToVoid => "ISO C does not allow indirection on operand of type 'void *'".to_string(),

            // 6.3.3.3
            Diagnostic::InvalidUnary(ty) => format!("invalid argument type '{}' to unary expression", ty.display(sema)),

            // 6.3.3.4
            Diagnostic::SizeofVoid => "invalid application of 'sizeof' to a void type".to_string(),
            Diagnostic::SizeofIncomplete(ty) => format!("invalid application of 'sizeof' to an incomplete type '{}'", ty.display(sema)),
            Diagnostic::SizeofFunction => "invalid application of 'sizeof' to a function type".to_string(),
            Diagnostic::SizeofBitfield => "invalid application of 'sizeof' to bit-field".to_string(),

            // 6.3.4
            Diagnostic::CastToNonScalar => "Conversion to non scalar type".to_string(),
            Diagnostic::CastOfNonScalar => "Conversion of non scalar type".to_string(),

            // 6.3.5
            Diagnostic::DivisionByZero => "division by zero is undefined".to_string(),
            Diagnostic::ModuloByZero =>  "remainder by zero is undefined".to_string(),
            Diagnostic::InvalidBinaryOperand(lhs, rhs) => format!("invalid operands to binary expression ('{}' and '{}')", lhs.display(sema), rhs.display(sema)),

            // 6.3.7
            Diagnostic::ShiftCountNegative => "shift count is negative".to_string(),
            Diagnostic::ShiftCountOutOfRange => "shift count >= width of type".to_string(),

            // 6.3.8
            Diagnostic::OrderedFunctionPointers(lhs, rhs) => format!("ordered comparison of function pointers ('{}' and '{}')", lhs.display(sema), rhs.display(sema)),
            Diagnostic::MixedCompletenessComparison(lhs, rhs) => format!("ordered comparison needs two complete or two incomplete pointee types ('{}' and '{}')", lhs.display(sema), rhs.display(sema)),

            // 6.3.9
            Diagnostic::InvalidComparison(lhs, rhs) => format!("comparison of distinct pointer types ('{}' and '{}')", lhs.display(sema), rhs.display(sema)),

            // 6.3.15
            Diagnostic::NotScalar(ty) => format!("used type '{}' where arithmetic or pointer type is required", ty.display(sema)),
            Diagnostic::IncompatibleOperands(lhs, rhs) => format!("incompatible operand types ('{}' and '{}')", lhs.display(sema), rhs.display(sema)),
            Diagnostic::PointerMismatch(lhs, rhs) => format!("pointer type mismatch ('{}' and '{}')", lhs.display(sema), rhs.display(sema)),

            // 6.3.16
            Diagnostic::AssignToRValue => "expression is not assignable".to_string(),
            Diagnostic::ConstAssignment(ty) => format!("cannot assign to variable with const-qualified type '{}'", ty.display(sema)),
            Diagnostic::ConstMemberAssignment(ty) => format!("cannot assign to '{}' because it has a const-qualified member", ty.display(sema)),
            Diagnostic::AssignmentDiscardedQualifiers(to, from) => format!("assigning to '{}' from '{}' discards qualifiers", to.display(sema), from.display(sema)),
            Diagnostic::AssignmentIncompatibleTypes(to, from) => format!("assignment to '{}' from incompatible pointer type '{}'", to.display(sema), from.display(sema)),

            // 6.4
            Diagnostic::ConstantOverflow => "overflow in constant expression".to_string(),
            Diagnostic::NonConstantExpression => "Non constant expression".to_string(),
            Diagnostic::NonIntegerConstantExpression => "Non integer constant expression".to_string(),

            // 6.5
            Diagnostic::EmptyDeclaration => "Declaration declares nothing".to_string(),
            Diagnostic::InvalidTypeSpecifier => "Invalid type specifier or combination thereof".to_string(),
            Diagnostic::IncompleteVariable(ty) => format!("variable has incomplete type '{}'", ty.display(sema)),

            // 6.5.1
            Diagnostic::MultipleStorageSpecifiers => "Multiple storage class declaration".to_string(),
            Diagnostic::BlockScopeNotExtern => "Function in block not declared as extern".to_string(),

            // 6.5.2.1
            Diagnostic::TagWithoutMember(kind) => format!("{kind} has no named member"),
            Diagnostic::InvalidMemberType(ty) => format!("field has incomplete or function type '{}'", ty.display(sema)),
            Diagnostic::NonIntBitFieldType => "Bit-field has non-integral type".to_string(),
            Diagnostic::NegativeBitFieldWidth(Some(name), width) => format!("bit-field '{}' has negative width ({width})", name.id.resolve()),
            Diagnostic::NegativeBitFieldWidth(None, width) => format!("anonymous bit-field has negative width ({width})"),
            Diagnostic::BitFieldWidthTooLarge(Some(name), width, bits) => format!("width of bit-field '{}' ({width} bits) exceeds the width of its type ({bits} bits)", name.id.resolve()),
            Diagnostic::BitFieldWidthTooLarge(None, width, bits) => format!("width of anonymous bit-field ({width} bits) exceeds the width of its type ({bits} bits)"),
            Diagnostic::ZeroWidthNamedBitField(name) => format!("named bit-field '{}' has zero width", name.id.resolve()),

            // 6.5.2.2
            Diagnostic::ForwardEnumReference(Some(name)) => format!("ISO C forbids forward references to enum '{}'", name.id.resolve()),
            Diagnostic::ForwardEnumReference(None) => "ISO C forbids forward references to 'enum' types".to_string(),

            Diagnostic::EnumeratorBadValue => "Enumerator value should be in int range".to_string(),

            // 6.5.3
            Diagnostic::DuplicateTypeQualifiers => "Duplicate type qualifiers".to_string(),

            // 6.5.4.2
            Diagnostic::NonIntArraySize => "Array len has non-integral type".to_string(),
            Diagnostic::NegativeArraySize => "size of array is negative".to_string(),
            Diagnostic::ZeroArraySize => "size of array is zero".to_string(),
            Diagnostic::InvalidElementType(ty) => format!("array has incomplete or function element type '{}'", ty.display(sema)),

            // 6.5.4.3
            Diagnostic::FunctionReturningArray(ty) => format!("function cannot return array type '{}'", ty.display(sema)),
            Diagnostic::FunctionReturningFunction(ty) => format!("function cannot return function type '{}'", ty.display(sema)),
            Diagnostic::VoidParameter => "Parameter shall not have void type".to_string(),
            Diagnostic::ParameterNotRegister => "Parameter shall only by declared with register storage".to_string(),
            Diagnostic::DuplicateParameterName => "Duplicate paramter identifier".to_string(),

            // 6.5.7
            Diagnostic::ArrayInitTooLong => "excess elements in array initializer".to_string(),
            Diagnostic::BlockScopeLinkageInitializer => "declaration of block scope identifier with linkage cannot have an initializer".to_string(),
            Diagnostic::NonConstantInitializer => "initializer element is not a compile-time constant".to_string(),
            Diagnostic::InitDiscardedQualifiers(to, from) => format!("initializing '{}' with an expression of type '{}' discards qualifiers", to.display(sema), from.display(sema)),
            Diagnostic::InitIncompatibleTypes(to, from) => format!("initialization of '{}' from incompatible pointer type '{}'", to.display(sema), from.display(sema)),

            // 6.6.1

            // 6.6.4
            Diagnostic::NonScalarStatement(ty) => format!("statement requires expression of scalar type ('{}' invalid)", ty.display(sema)),

            // 6.6.4.2
            Diagnostic::NonIntegralStatement(ty) => format!("statement requires expression of integer type ('{}' invalid)", ty.display(sema)),

            // 6.6.6.4
            Diagnostic::ReturnWithoutValue => "'return' with no value, in function returning non-void".to_string(),
            Diagnostic::ReturnDiscardedQualifiers(to, from) => format!("returning '{}' from a function with result type '{}' discards qualifiers", from.display(sema), to.display(sema)),
            Diagnostic::ReturnIncompatibleTypes(to, from) => format!("returning '{}' from a function with incompatible result type '{}'", from.display(sema), to.display(sema)),

            // 6.7
            Diagnostic::AutoRegisterExternal => "External declaration auto of register".to_string(),
            Diagnostic::InternalNeverDefined(name) => format!("'{}' used but never defined", name.id.resolve()),
            Diagnostic::TentativeNeverCompleted(ty) => format!("tentative definition has type '{}' that is never completed", ty.display(sema)),

            // 6.7.1
            Diagnostic::NotFunctionTypeDeclarator => "Declarator shall be function type".to_string(),
            Diagnostic::FunctionAutoExtern => "Function storage shall be auto or extern".to_string(),
            Diagnostic::UnnamedPrototypeParameter => "Parameter shall include an identifier".to_string(),
            Diagnostic::ParameterTypeListWithList => "Parameter style function declration shall not be followed by a declaration list".to_string(),
            Diagnostic::MissingParameterInOldStyle => "Missing parameter".to_string(),
            Diagnostic::IncompleteParameter(ty) => format!("parameter has incomplete type '{}'", ty.display(sema)),
            Diagnostic::IncompleteReturn(ty) => format!("function definition has incomplete return type '{}'", ty.display(sema)),
        }
    }
}

pub trait DiagnosticSink {
    fn diagnostics(&mut self) -> &mut Vec<DiagnosticNode>;

    fn add_diag<T>(&mut self, diag: Diag<T>, span: &Span) -> T {
        match diag.diagnostic {
            Some(Diagnostic::Poisoned) | None => (),
            Some(diagnostic) => self.diagnostics().push(DiagnosticNode::new(diagnostic, *span)),
        }
        diag.res
    }
}

#[derive(Debug)]
pub struct Diag<T> {
    pub res: T,
    pub diagnostic: Option<Diagnostic>,
}

impl<T> Diag<T> {
    pub fn collect<C: DiagnosticSink>(self, collector: &mut C, span: &Span) -> T {
        collector.add_diag(self, span)
    }

    pub fn new(res: T, diagnostic: Option<Diagnostic>) -> Self {
        Self { res, diagnostic }
    }

    pub fn ok(res: T) -> Self {
        Self::new(res, None)
    }

    pub fn err(res: T, diagnostic: Diagnostic) -> Self {
        Self::new(res, Some(diagnostic))
    }

    pub fn into_result(self) -> Result<T, Diagnostic> {
        match self.diagnostic {
            Some(diagnostic) => Err(diagnostic),
            None => Ok(self.res),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticNode {
    pub span: Span,
    pub inner: Diagnostic,
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
        Self { names: buf, len: names.len() }
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
