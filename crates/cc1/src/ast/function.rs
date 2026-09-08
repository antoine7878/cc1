use crate::ast::{DeclarationSpecifier, DeclaratorNode, Name, TypeSpecifier};
use crate::ast_node;
use crate::context::Context;
use libft::Span;

ast_node! {
    pub struct FunctionParametersNode {
        pub param: FunctionParameters,
    }
}

#[derive(Clone, Debug, PartialEq)]
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
    pub fn is_abstract_void(&self, ctx: &Context) -> bool {
        matches!(
            self.specifiers.as_slice(),
            [DeclarationSpecifier::Type(TypeSpecifier::Void)]
        ) && self.declarator.is_abstract(ctx)
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
