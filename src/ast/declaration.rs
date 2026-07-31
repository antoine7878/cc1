use crate::parser::Span;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Declaration {
    span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Initializer;
