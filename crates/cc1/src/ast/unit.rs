use crate::ast::{CompoundStatementNode, DeclarationNode, DeclarationSpecifier, DeclaratorNode};
use crate::ast_node;
use libft::Span;

ast_node! {
    pub struct TranslationUnitNode {
        pub declarations: Vec<ExternalDeclarationNode>,
    }
}

impl Default for TranslationUnitNode {
    fn default() -> Self {
        TranslationUnitNode::new(Vec::new(), Span::default())
    }
}

ast_node! {
    pub struct ExternalDeclarationNode {
        pub decl: ExternalDeclaration,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExternalDeclaration {
    Function(FunctionDefinitionNode),
    Declaration(DeclarationNode),
}

impl ExternalDeclarationNode {
    pub fn declaration(decl: DeclarationNode, span: Span) -> ExternalDeclarationNode {
        ExternalDeclarationNode::new(ExternalDeclaration::Declaration(decl), span)
    }

    pub fn function(decl: FunctionDefinitionNode, span: Span) -> ExternalDeclarationNode {
        ExternalDeclarationNode::new(ExternalDeclaration::Function(decl), span)
    }
}

ast_node! {
    pub struct FunctionDefinitionNode {
        pub specifiers: Vec<DeclarationSpecifier>,
        pub declarator: DeclaratorNode,
        pub old_style_declarations: Vec<DeclarationNode>,
        pub body: CompoundStatementNode,
    }
}
