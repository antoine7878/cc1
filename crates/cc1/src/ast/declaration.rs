use libft::Span;

use crate::ast::{DeclarationSpecifier, ExpressionNode, FunctionParametersNode, Name, Qualifier};
use crate::{ast_node, define_arena};

define_arena!(Declarator, DeclaratorArena, DeclaratorId);

ast_node! {
    pub struct DeclarationNode {
        pub specifiers: Vec<DeclarationSpecifier>,
        pub init_declarators: Vec<InitDeclaratorNode>,
    }
}

ast_node! {
    pub struct InitDeclaratorNode {
        pub declarator: DeclaratorNode,
        pub initializer: Option<InitializerNode>,
    }
}

ast_node! {
    pub struct DeclaratorNode {
        pub id: DeclaratorId,
    }
}

impl DeclaratorNode {
    pub fn ident(&self) -> Option<Name> {
        self.id.resolve().ident()
    }

    pub fn is_abstract(&self) -> bool {
        matches!(self.id.resolve(), Declarator::Abstract)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declarator {
    Ident(Name),
    Abstract,
    Pointer { qualifiers: Vec<Qualifier>, inner: DeclaratorNode },
    Array { declarator: DeclaratorNode, size: Option<ExpressionNode> },
    Function { declarator: DeclaratorNode, params: FunctionParametersNode },
}

impl Declarator {
    pub fn ident(&self) -> Option<Name> {
        match self {
            Declarator::Ident(n) => Some(*n),
            Declarator::Abstract => None,
            Declarator::Pointer { inner: declarator, .. }
            | Declarator::Array { declarator, .. }
            | Declarator::Function { declarator, .. } => declarator.ident(),
        }
    }
}

ast_node! {
    pub struct InitializerNode {
        pub init: Initializer,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Initializer {
    Single(ExpressionNode),
    List(Vec<InitializerNode>),
}

impl DeclaratorArena {
    fn add(&mut self, id: Declarator, span: Span) -> DeclaratorNode {
        DeclaratorNode { id: self.alloc(id), span }
    }

    pub fn abstract_declarator(&mut self, span: Span) -> DeclaratorNode {
        self.add(Declarator::Abstract, span)
    }

    pub fn ident(&mut self, ident: Name, span: Span) -> DeclaratorNode {
        self.add(Declarator::Ident(ident), span)
    }

    pub fn pointer(&mut self, qualifiers: Vec<Qualifier>, inner: DeclaratorNode, span: Span) -> DeclaratorNode {
        self.add(Declarator::Pointer { qualifiers, inner }, span)
    }

    pub fn with_pointer(&mut self, pointer: Vec<Vec<Qualifier>>, decl: DeclaratorNode, span: Span) -> DeclaratorNode {
        pointer.into_iter().fold(decl, |acc, qs| self.pointer(qs, acc, span))
    }

    pub fn array(&mut self, declarator: DeclaratorNode, size: Option<ExpressionNode>, span: Span) -> DeclaratorNode {
        self.add(Declarator::Array { declarator, size }, span)
    }

    pub fn function(
        &mut self,
        declarator: DeclaratorNode,
        params: FunctionParametersNode,
        span: Span,
    ) -> DeclaratorNode {
        self.add(Declarator::Function { declarator, params }, span)
    }
}
