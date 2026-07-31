// ==================================================================
// Top-level translation unit
// ==================================================================
pub struct TranslationUnit {
    pub decls: Vec<ExternalDeclaration>,
}

pub enum ExternalDeclaration {
    FunctionDefinition(FunctionDefinition),
    Declaration(Declaration),
    EmptyDeclaration, // lone ';'
}

// ==================================================================
// Function definition
// ==================================================================
pub struct FunctionDefinition {
    pub specifiers: Vec<Specifier>,
    pub declarator: Declarator, // must be a Function declarator
    // K&R‑style parameter declarations (empty for prototype style)
    pub old_style_parms: Vec<Declaration>,
    pub body: CompoundStatement,
}

// ==================================================================
// Declarations
// ==================================================================
pub struct Declaration {
    pub specifiers: Vec<Specifier>,
    pub init_declarators: Vec<InitDeclarator>,
}

pub struct InitDeclarator {
    pub declarator: Declarator,
    pub initializer: Option<Initializer>,
}

// Declarator – handles named and abstract declarators
pub enum Declarator {
    Ident(String),                    // name (empty string => abstract)
    Pointer(Box<Declarator>),
    Array {
        declarator: Box<Declarator>,
        size: Option<Box<Expression>>, // missing => [] (incomplete / outer array)
    },
    Function {
        declarator: Box<Declarator>,
        params: FunctionParameters,
    },
}

pub enum FunctionParameters {
    /// Old‑style K&R: `f(a,b)` or `f()`  (empty vec => `()`)
    OldStyle(Vec<String>),
    /// Prototype style: `f(void)` (empty vec), `f(int a, int b)`
    ParameterTypeList(Vec<ParameterDeclaration>),
}

pub struct ParameterDeclaration {
    pub specifiers: Vec<Specifier>,
    pub declarator: Declarator, // may be abstract (name = "")
}

// ==================================================================
// Specifiers – storage class, type specifiers, qualifiers
// ==================================================================
pub enum Specifier {
    StorageClass(StorageClass),
    TypeSpec(TypeSpecifier),
    Qualifier(TypeQualifier),
}

pub enum StorageClass {
    Typedef,
    Extern,
    Static,
    Auto,
    Register,
}

pub enum TypeQualifier {
    Const,
    Volatile,
}

pub enum TypeSpecifier {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    Struct(StructSpec),
    Union(UnionSpec),
    Enum(EnumSpec),
    TypedefName(String),
}

// ==================================================================
// Struct / union / enum
// ==================================================================
pub struct StructSpec {
    pub name: Option<String>,
    pub fields: Vec<StructDeclaration>,
}

pub struct UnionSpec {
    pub name: Option<String>,
    pub fields: Vec<StructDeclaration>,
}

pub struct StructDeclaration {
    pub specifiers: Vec<Specifier>, // only type spec & qualifier
    pub struct_declarators: Vec<StructDeclarator>,
}

pub struct StructDeclarator {
    pub declarator: Declarator, // may be abstract
    pub bit_width: Option<Expression>,
}

pub struct EnumSpec {
    pub name: Option<String>,
    pub enumerators: Vec<Enumerator>,
}

pub struct Enumerator {
    pub name: String,
    pub value: Option<Expression>,
}

// ==================================================================
// Expressions
// ==================================================================
pub enum Expression {
    Identifier(String),
    Constant(Constant),
    StringLiteral(String),
    Paren(Box<Expression>),

    // Unary
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
    UnaryPlus(Box<Expression>),
    UnaryMinus(Box<Expression>),
    BitNot(Box<Expression>),
    LogicalNot(Box<Expression>),
    PreIncrement(Box<Expression>),
    PreDecrement(Box<Expression>),
    PostIncrement(Box<Expression>),
    PostDecrement(Box<Expression>),
    SizeofExpr(Box<Expression>),
    SizeofType(Type),
    Cast(Type, Box<Expression>),

    // Binary arithmetic
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),

    // Binary bitwise
    BitAnd(Box<Expression>, Box<Expression>),
    BitOr(Box<Expression>, Box<Expression>),
    BitXor(Box<Expression>, Box<Expression>),
    Shl(Box<Expression>, Box<Expression>),
    Shr(Box<Expression>, Box<Expression>),

    // Comparisons
    Eq(Box<Expression>, Box<Expression>),
    Neq(Box<Expression>, Box<Expression>),
    Lt(Box<Expression>, Box<Expression>),
    Le(Box<Expression>, Box<Expression>),
    Gt(Box<Expression>, Box<Expression>),
    Ge(Box<Expression>, Box<Expression>),

    // Logical
    LogicalAnd(Box<Expression>, Box<Expression>),
    LogicalOr(Box<Expression>, Box<Expression>),

    // Assignment
    Assign(Box<Expression>, Box<Expression>),
    AddAssign(Box<Expression>, Box<Expression>),
    SubAssign(Box<Expression>, Box<Expression>),
    MulAssign(Box<Expression>, Box<Expression>),
    DivAssign(Box<Expression>, Box<Expression>),
    ModAssign(Box<Expression>, Box<Expression>),
    BitAndAssign(Box<Expression>, Box<Expression>),
    BitOrAssign(Box<Expression>, Box<Expression>),
    BitXorAssign(Box<Expression>, Box<Expression>),
    ShlAssign(Box<Expression>, Box<Expression>),
    ShrAssign(Box<Expression>, Box<Expression>),

    Comma(Box<Expression>, Box<Expression>),

    // Ternary
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },

    // Function call
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },

    // Array subscript
    Subscript {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    // Member access
    Member {
        object: Box<Expression>,
        field: String,
    },
    PtrMember {
        object: Box<Expression>,
        field: String,
    },
}

pub enum Constant {
    Integer(String),  // raw token, e.g. "42", "0x1A"
    Float(String),    // raw token, e.g. "3.14", "1e-5"
    Character(u32),   // code point value (e.g. 'A' → 65)
}

// ==================================================================
// Initializers
// ==================================================================
pub enum Initializer {
    Expr(Expression),
    List(Vec<Initializer>),
}

// ==================================================================
// Statements
// ==================================================================
pub enum Statement {
    Compound(CompoundStatement),
    Expr(Option<Expression>), // `None` → empty statement (`;`)
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Switch {
        condition: Expression,
        body: Box<Statement>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    For {
        init: Option<Expression>,       // C89 allows only an expression here
        condition: Option<Expression>,
        iteration: Option<Expression>,
        body: Box<Statement>,
    },
    Goto(String),
    Continue,
    Break,
    Return(Option<Expression>),
    Labeled {
        label: Label,
        body: Box<Statement>,
    },
}

pub enum Label {
    Ident(String),
    Case(Expression),
    Default,
}

pub struct CompoundStatement {
    pub decls: Vec<Declaration>,   // C89: all declarations must precede statements
    pub stmts: Vec<Statement>,
}

// ==================================================================
// Type node (used by sizeof(type), cast, etc.)
// ==================================================================
pub struct Type {
    pub specifiers: Vec<Specifier>,
    pub declarator: Declarator, // abstract declarator (Identifier(""))
}
