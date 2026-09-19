use libft::Span;

use crate::ast::{DeclarationSpecifier, DeclaratorNode, ExpressionNode, Name};
use crate::{ast_node, define_arena};

define_arena!(Struct, StructArena, StructId);
define_arena!(Union, UnionArena, UnionId);
define_arena!(Enum, EnumArena, EnumId);
define_arena!(Enumerator, EnumeratorArena, EnumeratorId);

ast_node! {
    pub struct Struct {
        pub name: Option<Name>,
        pub declarations: Vec<StructDeclaration>,
    }
}

ast_node! {
    pub struct Union {
        pub name: Option<Name>,
        pub declarations: Vec<StructDeclaration>,
    }
}

ast_node! {
    pub struct StructDeclaration {
        pub specifiers: Vec<DeclarationSpecifier>,
        pub struct_declarators: Vec<StructMemberDeclarator>,
    }
}

ast_node! {
    pub struct StructMemberDeclarator {
        pub declarator: DeclaratorNode,
        pub bit_width: Option<ExpressionNode>,
    }
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub span: Span,
    pub name: Option<Name>,
    pub enumerators: Vec<EnumeratorId>,
}

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub span: Span,
    pub name: Name,
    pub value: Option<ExpressionNode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Struct,
    Union,
    Enum,
}
impl StructArena {
    pub fn add(&mut self, name: Option<Name>, declarations: Vec<StructDeclaration>, span: Span) -> StructId {
        self.alloc(Struct { name, declarations, span })
    }
}

impl UnionArena {
    pub fn add(&mut self, name: Option<Name>, declarations: Vec<StructDeclaration>, span: Span) -> UnionId {
        self.alloc(Union { name, declarations, span })
    }
}

impl EnumArena {
    pub fn add(&mut self, name: Option<Name>, enumerators: Vec<EnumeratorId>, span: Span) -> EnumId {
        self.alloc(Enum { name, enumerators, span })
    }
}

impl EnumeratorArena {
    pub fn add(&mut self, name: Name, value: Option<ExpressionNode>, span: Span) -> EnumeratorId {
        self.alloc(Enumerator { name, value, span })
    }
}
