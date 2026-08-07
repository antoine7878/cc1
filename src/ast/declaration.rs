use std::fmt::Display;

use crate::arena::{Arena, ArenaId};
use crate::ast::{DeclarationSpecifier, ExpressionNode, FunctionParametersNode, Name, Qualifier};
use crate::define_arena;
use crate::parser::Span;

define_arena!(Declarator, DeclaratorArena, DeclaratorId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeclarationNode {
    pub span: Span,
    pub specifiers: Vec<DeclarationSpecifier>,
    pub init_declarators: Vec<InitDeclaratorNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InitDeclaratorNode {
    pub span: Span,
    pub declarator: DeclaratorNode,
    pub initializer: Option<InitializerNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeclaratorNode {
    pub span: Span,
    pub id: DeclaratorId,
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InitializerNode {
    pub span: Span,
    pub init: Initializer,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Initializer {
    Single(ExpressionNode),
    List(Vec<InitializerNode>),
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

impl InitDeclaratorNode {
    pub fn new(declarator: DeclaratorNode, initializer: Option<InitializerNode>, span: Span) -> Self {
        Self {
            declarator,
            initializer,
            span,
        }
    }
}

impl DeclaratorNode {
    pub fn new(id: DeclaratorId, span: Span) -> Self {
        Self { id, span }
    }
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

impl InitializerNode {
    pub fn new(init: Initializer, span: Span) -> Self {
        Self { init, span }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ExpressionId, StringArena};

    fn name(s: &str) -> Name {
        let mut arena = StringArena::default();
        arena.add(s.to_string(), Span::default())
    }

    fn declarator() -> DeclaratorNode {
        DeclaratorNode {
            id: DeclaratorId::from(0),
            span: Span::default(),
        }
    }

    #[test]
    fn display_declarators() {
        let d = declarator();
        let cases = [
            (Declarator::Ident(name("x")), "Ident"),
            (Declarator::Abstract, "Abstract"),
            (
                Declarator::Pointer {
                    qualifiers: vec![],
                    inner: Some(d.clone()),
                },
                "Pointer",
            ),
            (
                Declarator::Array {
                    declarator: d.clone(),
                    size: None,
                },
                "Array",
            ),
            (
                Declarator::Function {
                    declarator: d.clone(),
                    params: FunctionParametersNode::empty(Span::default()),
                },
                "Function",
            ),
        ];
        for (kind, expect) in cases {
            assert_eq!(kind.to_string(), expect);
        }
    }

    #[test]
    fn display_initializers() {
        let expr = ExpressionNode {
            span: Span::default(),
            id: ExpressionId::from(0),
        };
        assert_eq!(Initializer::Single(expr).to_string(), "Single");
        assert_eq!(Initializer::List(vec![]).to_string(), "List");
    }
}
