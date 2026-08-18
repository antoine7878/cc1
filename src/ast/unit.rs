use crate::ast::{CompoundStatementNode, DeclarationNode, DeclarationSpecifier, DeclaratorNode, Node};
use crate::ast_node;
use crate::parser::{Position, Span};

ast_node! {
    pub struct TranslationUnitNode {
        pub declarations: Vec<ExternalDeclarationNode>,
    }
}

impl Default for TranslationUnitNode {
    fn default() -> Self {
        Self {
            span: Span::new(Position { line: 0, col: 0 }, Position { line: 0, col: 0 }),
            declarations: Vec::new(),
        }
    }
}

ast_node! {
    pub struct ExternalDeclarationNode {
        pub decl: ExternalDeclaration,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ExternalDeclaration {
    Function(FunctionDefinitionNode),
    Declaration(DeclarationNode),
}

impl ExternalDeclarationNode {
    pub fn declaration(decl: DeclarationNode, span: Span) -> ExternalDeclarationNode {
        ExternalDeclarationNode {
            span,
            decl: ExternalDeclaration::Declaration(decl),
        }
    }

    pub fn function(decl: FunctionDefinitionNode, span: Span) -> ExternalDeclarationNode {
        ExternalDeclarationNode {
            span,
            decl: ExternalDeclaration::Function(decl),
        }
    }
}

ast_node! {
    pub struct FunctionDefinitionNode {
        pub specifiers: Vec<DeclarationSpecifier>,
        pub declarator: DeclaratorNode,
        pub arguments: Vec<DeclarationNode>,
        pub body: CompoundStatementNode,
    }
}
