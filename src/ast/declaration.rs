use crate::ast::ExpressionNode;
use crate::ast::{Qualifier, TypeNode, symbol::Storage};
use crate::parser::Span;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeclarationNode {
    pub span: Span,
    pub specifiers: Vec<DeclarationSpecifier>,
    pub init_declarators: Vec<InitDeclaratorNode>,
}

impl DeclarationNode {
    pub fn new(specifiers: Vec<DeclarationSpecifier>, init_declarators: Vec<InitDeclaratorNode>, span: Span) -> Self {
        Self {
            specifiers,
            init_declarators,
            span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DeclarationSpecifier {
    Type(TypeNode),
    Qualifier(Qualifier),
    Storage(Storage),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InitDeclaratorNode {
    pub span: Span,
    pub init_declarator: DeclaratorNode,
    pub initializer: Option<InitializerNode>,
}

impl InitDeclaratorNode {
    pub fn new(init_declarator: DeclaratorNode, initializer: Option<InitializerNode>, span: Span) -> Self {
        Self {
            init_declarator,
            initializer,
            span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeclaratorNode {
    span: Span,
    declarator: Declarator,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Declarator {
    Ident(String),
    Pointer(Box<Declarator>),
    Array {
        declarator: Box<Declarator>,
        size: Option<ExpressionNode>,
    },
    Function {
        declarator: Box<Declarator>,
        // params: FunctionParameters,
    },
}

impl DeclaratorNode {
    pub fn new(declarator: Declarator, span: Span) -> Self {
        Self { declarator, span }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InitializerNode {
    span: Span,
    init: Initializer,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Initializer {
    Single(ExpressionNode),
    List(Vec<ExpressionNode>),
}
impl InitializerNode {
    pub fn new(init: Initializer, span: Span) -> Self {
        Self { init, span }
    }
}
