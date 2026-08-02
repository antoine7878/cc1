use crate::ast::{DeclarationSpecifier, DeclaratorNode, Name};
use crate::parser::Span;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FunctionParametersNode {
    span: Span,
    param: FunctionParameters,
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
            param: FunctionParameters::ParameterTypeList(params),
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
