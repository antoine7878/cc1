use std::fmt::Display;

use crate::ast::{DeclarationSpecifier, ExpressionNode, FunctionParametersNode, Name, Node, Qualifier};
use crate::parser::{Context, Span};
use crate::{ast_node, define_arena};

define_arena!(Declarator, DeclaratorArena, DeclaratorId, declarators);

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
    pub fn ident(&self, ctx: &Context) -> Option<Name> {
        self.id.resolve(&ctx.arenas).ident(ctx)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Declarator {
    Ident(Name),
    Abstract,
    Pointer {
        qualifiers: Vec<Qualifier>,
        inner: Option<DeclaratorNode>,
    },
    Array {
        declarator: DeclaratorNode,
        size: Option<ExpressionNode>,
    },
    Function {
        declarator: DeclaratorNode,
        params: FunctionParametersNode,
    },
}

impl Declarator {
    pub fn ident(&self, ctx: &Context) -> Option<Name> {
        match self {
            Declarator::Ident(n) => Some(n.clone()),
            Declarator::Abstract | Declarator::Pointer { inner: None, .. } => None,
            Declarator::Pointer {
                inner: Some(declarator),
                ..
            }
            | Declarator::Array { declarator, .. }
            | Declarator::Function { declarator, .. } => declarator.ident(ctx),
        }
    }
}

ast_node! {
    pub struct InitializerNode {
        pub init: Initializer,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Initializer {
    Single(ExpressionNode),
    List(Vec<InitializerNode>),
}

impl DeclaratorArena {
    fn add(&mut self, id: Declarator, span: Span) -> DeclaratorNode {
        DeclaratorNode {
            id: self.alloc(id),
            span,
        }
    }

    pub fn abstrct(&mut self, span: Span) -> DeclaratorNode {
        self.add(Declarator::Abstract, span)
    }

    pub fn ident(&mut self, ident: Name, span: Span) -> DeclaratorNode {
        self.add(Declarator::Ident(ident), span)
    }

    pub fn pointer(&mut self, qualifiers: Vec<Qualifier>, inner: Option<DeclaratorNode>, span: Span) -> DeclaratorNode {
        self.add(Declarator::Pointer { qualifiers, inner }, span)
    }

    pub fn with_pointer(&mut self, pointer: DeclaratorNode, i: DeclaratorNode, span: Span) -> DeclaratorNode {
        match self.get(pointer.id).clone() {
            Declarator::Pointer {
                qualifiers,
                inner: None,
            } => self.pointer(qualifiers.clone(), Some(i), span),
            Declarator::Pointer {
                qualifiers,
                inner: Some(a),
            } => {
                let b = self.with_pointer(a.clone(), i, span);
                self.pointer(qualifiers.clone(), Some(b), span)
            }
            _ => unreachable!(),
        }
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

impl Display for Declarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Declarator::Ident(_) => "Ident",
            Declarator::Abstract => "Abstract",
            Declarator::Pointer { .. } => "Pointer",
            Declarator::Array { .. } => "Array",
            Declarator::Function { .. } => "Function",
        };
        write!(f, "{}", s)
    }
}

impl Display for Initializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Initializer::Single(_) => "Single",
            Initializer::List(_) => "List",
        };
        write!(f, "{}", s)
    }
}
