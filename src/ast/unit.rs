use crate::{
    ast::{CompoundStatementNode, DeclarationNode, DeclarationSpecifier, DeclaratorNode},
    parser::Span,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TranslationUnitNode {
    pub span: Span,
    pub declarations: Vec<ExternalDeclarationNode>,
}

impl TranslationUnitNode {
    pub fn new(declarations: Vec<ExternalDeclarationNode>, span: Span) -> TranslationUnitNode {
        TranslationUnitNode { declarations, span }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ExternalDeclarationNode {
    pub span: Span,
    pub decl: ExternalDeclaration,
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FunctionDefinitionNode {
    pub span: Span,
    pub specifiers: Vec<DeclarationSpecifier>,
    pub declarator: DeclaratorNode,
    pub declarations: Vec<DeclarationNode>,
    pub coumpound: CompoundStatementNode,
}

impl FunctionDefinitionNode {
    pub fn new(
        specifiers: Vec<DeclarationSpecifier>,
        declarator: DeclaratorNode,
        declarations: Vec<DeclarationNode>,
        coumpound: CompoundStatementNode,
        span: Span,
    ) -> FunctionDefinitionNode {
        FunctionDefinitionNode {
            span,
            specifiers,
            declarator,
            declarations,
            coumpound,
        }
    }
}
