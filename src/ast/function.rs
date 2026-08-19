use std::fmt::Display;

use crate::ast::{DeclarationSpecifier, DeclaratorNode, Name, TypeSpecifier};
use crate::ast_node;
use crate::parser::{Context, Span};

ast_node! {
    pub struct FunctionParametersNode {
        pub param: FunctionParameters,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum FunctionParameters {
    Empty,
    OldStyle(Vec<Name>),
    ParameterTypeList(Vec<ParameterDeclaration>),
    Variadic(Vec<ParameterDeclaration>),
}

ast_node! {
    pub struct ParameterDeclaration {
        pub specifiers: Vec<DeclarationSpecifier>,
        pub declarator: DeclaratorNode,
    }
}

impl ParameterDeclaration {
    fn is_abstract_void(&self, ctx: &Context) -> bool {
        if !matches!(
            self.specifiers.as_slice(),
            [DeclarationSpecifier::Type(TypeSpecifier::Void)]
        ) {
            return false;
        }
        true
    }
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
