use crate::arena::{Arena, ArenaId};
use crate::ast::{DeclarationSpecifier, Declarator, ExpressionNode, StringId};
use crate::define_arena;
use crate::parser::Span;

define_arena!(Struct, StructArena, StructId);
define_arena!(Union, UnionArena, UnionId);
define_arena!(Enum, EnumArena, EnumId);
define_arena!(Variant, VariantArena, VariantId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Struct {
    pub span: Span,
    pub name: Option<StringId>,
    pub fields: Vec<StructDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Union {
    pub span: Span,
    pub name: Option<StringId>,
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
    pub declarator: Declarator,
    pub bit_width: Option<ExpressionNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Enum {
    pub span: Span,
    pub name: Option<StringId>,
    pub variants: Vec<VariantId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Variant {
    pub span: Span,
    pub name: StringId,
    pub value: Option<ExpressionNode>,
}

impl StructArena {
    pub fn add(&mut self, name: Option<StringId>, fields: Vec<StructDeclaration>, span: Span) -> StructId {
        self.alloc(Struct { name, fields, span })
    }
}

impl UnionArena {
    pub fn add(&mut self, name: Option<StringId>, fields: Vec<StructDeclaration>, span: Span) -> UnionId {
        self.alloc(Union { name, fields, span })
    }
}

impl EnumArena {
    pub fn add(&mut self, name: Option<StringId>, variants: Vec<VariantId>, span: Span) -> EnumId {
        self.alloc(Enum { name, variants, span })
    }
}

impl VariantArena {
    pub fn add(&mut self, name: StringId, value: Option<ExpressionNode>, span: Span) -> VariantId {
        self.alloc(Variant { name, value, span })
    }
}
