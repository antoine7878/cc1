use crate::ast::declaration::Declaration;
use crate::parser::Span;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TranslationUnit {
    span: Span,
    external_declarations: Vec<ExternalDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ExternalDeclaration {
    FunctionDefinition(FunctionDefinition),
    Declaration(Declaration),
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FunctionDefinition {
    span: Span,
}

// #[derive(Clone, Debug, Eq, PartialEq, Hash)]
// pub struct Statement {
//     span: Span,
// }

// #[derive(Clone, Debug, Eq, PartialEq, Hash)]
// pub struct Node {
//     span: Span,
//     node: NodeKind,
// }

// #[derive(Clone, Debug, Eq, PartialEq, Hash)]
// pub enum NodeKind {
//     Expression(Expression),
//     Declaration(Declaration),
//     TranslationUnit(TranslationUnit),
// }
