use crate::arena::{Arena, ArenaId};
use crate::ast::{DeclarationSpecifier, DeclaratorNode, ExpressionNode, Name};
use crate::define_arena;
use crate::parser::Span;

define_arena!(Struct, StructArena, StructId);
define_arena!(Union, UnionArena, UnionId);
define_arena!(Enum, EnumArena, EnumId);
define_arena!(Variant, VariantArena, VariantId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Struct {
    pub span: Span,
    pub name: Option<Name>,
    pub fields: Vec<StructDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Union {
    pub span: Span,
    pub name: Option<Name>,
    pub fields: Vec<StructDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StructDeclaration {
    pub span: Span,
    pub specifiers: Vec<DeclarationSpecifier>,
    pub struct_declarators: Vec<StructDeclarator>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StructDeclarator {
    pub span: Span,
    pub declarator: DeclaratorNode,
    pub bit_width: Option<ExpressionNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Enum {
    pub span: Span,
    pub name: Option<Name>,
    pub variants: Vec<VariantId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Variant {
    pub span: Span,
    pub name: Name,
    pub value: Option<ExpressionNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Tag {
    Struct,
    Union,
    Enum,
}

impl StructDeclaration {
    pub fn new(
        specifiers: Vec<DeclarationSpecifier>,
        struct_declarators: Vec<StructDeclarator>,
        span: Span,
    ) -> StructDeclaration {
        StructDeclaration {
            specifiers,
            struct_declarators,
            span,
        }
    }
}

impl StructDeclarator {
    pub fn new(declarator: DeclaratorNode, bit_width: Option<ExpressionNode>, span: Span) -> StructDeclarator {
        StructDeclarator {
            declarator,
            bit_width,
            span,
        }
    }
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
