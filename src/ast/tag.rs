use crate::ast::{AstArenas, DeclarationSpecifier, DeclaratorNode, ExpressionNode, Name};
use crate::parser::Span;
use crate::{ast_node, define_arena};

define_arena!(Struct, StructArena, StructId, AstArenas, arenas, structs);
define_arena!(Union, UnionArena, UnionId, AstArenas, arenas, unions);
define_arena!(Enum, EnumArena, EnumId, AstArenas, arenas, enums);
define_arena!(Variant, VariantArena, VariantId, AstArenas, arenas, variants);

ast_node! {
    pub struct Struct {
        pub name: Option<Name>,
        pub fields: Vec<StructDeclaration>,
    }
}

ast_node! {
    pub struct Union {
        pub name: Option<Name>,
        pub fields: Vec<StructDeclaration>,
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
    pub variants: Vec<VariantId>,
}

#[derive(Clone, Debug)]
pub struct Variant {
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
    pub fn add(&mut self, name: Option<Name>, fields: Vec<StructDeclaration>, span: Span) -> StructId {
        self.alloc(Struct { name, fields, span })
    }
}

impl UnionArena {
    pub fn add(&mut self, name: Option<Name>, fields: Vec<StructDeclaration>, span: Span) -> UnionId {
        self.alloc(Union { name, fields, span })
    }
}

impl EnumArena {
    pub fn add(&mut self, name: Option<Name>, variants: Vec<VariantId>, span: Span) -> EnumId {
        self.alloc(Enum { name, variants, span })
    }
}

impl VariantArena {
    pub fn add(&mut self, name: Name, value: Option<ExpressionNode>, span: Span) -> VariantId {
        self.alloc(Variant { name, value, span })
    }
}
