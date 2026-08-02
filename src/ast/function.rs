use std::fmt::Display;

use crate::ast::{DeclarationSpecifier, DeclaratorNode, Name};
use crate::parser::Span;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FunctionParametersNode {
    pub span: Span,
    pub param: FunctionParameters,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum FunctionParameters {
    Empty,
    OldStyle(Vec<Name>),
    ParameterTypeList(Vec<ParameterDeclaration>),
    Variadic(Vec<ParameterDeclaration>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ParameterDeclaration {
    pub span: Span,
    pub specifiers: Vec<DeclarationSpecifier>,
    pub declarator: DeclaratorNode,
}

impl FunctionParametersNode {
    pub fn empty(span: Span) -> FunctionParametersNode {
        FunctionParametersNode {
            span,
            param: FunctionParameters::Empty,
        }
    }

    pub fn old_style(names: Vec<Name>, span: Span) -> FunctionParametersNode {
        FunctionParametersNode {
            span,
            param: FunctionParameters::OldStyle(names),
        }
    }

    pub fn param_style(params: Vec<ParameterDeclaration>, span: Span) -> FunctionParametersNode {
        FunctionParametersNode {
            span,
            param: FunctionParameters::ParameterTypeList(params),
        }
    }

    pub fn variadic(params: Vec<ParameterDeclaration>, span: Span) -> FunctionParametersNode {
        FunctionParametersNode {
            span,
            param: FunctionParameters::Variadic(params),
        }
    }
}

impl ParameterDeclaration {
    pub fn new(specifiers: Vec<DeclarationSpecifier>, declarator: DeclaratorNode, span: Span) -> ParameterDeclaration {
        ParameterDeclaration {
            specifiers,
            declarator,
            span,
        }
    }
}

impl Display for FunctionParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FunctionParameters::Empty => "Empty",
            FunctionParameters::OldStyle(_) => "OldStyle",
            FunctionParameters::ParameterTypeList(_) => "Parameters",
            FunctionParameters::Variadic(_) => "Variadic",
        };
        write!(f, "{}", s)
    }
}
