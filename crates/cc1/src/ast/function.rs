use libft::Span;

use crate::ast::{DeclarationSpecifier, DeclaratorNode, Name, TypeSpecifier};
use crate::ast_node;
use crate::semantic::Resolver;

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
    pub fn is_abstract_void(&self, resolver: &mut Resolver) -> bool {
        if !self.declarator.is_abstract() {
            return false;
        }
        let [DeclarationSpecifier::Type(s)] = self.specifiers.as_slice() else { return false };
        if matches!(s, TypeSpecifier::Void) {
            return true;
        }
        let TypeSpecifier::TypedefName(name) = s else { return false };
        let Some(sym) = resolver.lookup_ordinary(name.id) else { return false };
        sym.resolve_with(resolver.sema).ty.is_void(resolver.sema)
    }
}

impl FunctionParametersNode {
    pub fn empty(span: Span) -> FunctionParametersNode {
        FunctionParametersNode { span, param: FunctionParameters::Empty }
    }

    pub fn old_style(names: Vec<Name>, span: Span) -> FunctionParametersNode {
        FunctionParametersNode { span, param: FunctionParameters::OldStyle(names) }
    }

    pub fn param_style(params: Vec<ParameterDeclaration>, span: Span) -> FunctionParametersNode {
        FunctionParametersNode { span, param: FunctionParameters::ParameterTypeList(params) }
    }

    pub fn variadic(params: Vec<ParameterDeclaration>, span: Span) -> FunctionParametersNode {
        FunctionParametersNode { span, param: FunctionParameters::Variadic(params) }
    }
}
