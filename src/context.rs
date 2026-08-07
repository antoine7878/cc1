use std::collections::HashSet;

use crate::ast::{DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorArena, DeclaratorNode, Name};
use crate::ast::{EnumArena, ExpressionArena, StatementArena, Storage, StringArena, StringId, StructArena};
use crate::ast::{StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, UnionArena, VariantArena};
use crate::parser::Span;

#[derive(Debug, Default)]
pub struct Arenas {
    pub names: StringArena,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub variants: VariantArena,
    pub expressions: ExpressionArena,
    pub declarators: DeclaratorArena,
    pub statements: StatementArena,
}

#[derive(Debug, Default)]
pub struct Context {
    pub typedefs: HashSet<StringId>,
    pub arenas: Arenas,
    pub ast: TranslationUnitNode,
}

impl Context {
    pub fn struct_or_union(
        &mut self,
        tag: Tag,
        name: Option<Name>,
        fields: Vec<StructDeclaration>,
        span: Span,
    ) -> TypeSpecifier {
        match tag {
            Tag::Struct => TypeSpecifier::Struct(self.arenas.structs.add(name, fields, span)),
            Tag::Union => TypeSpecifier::Union(self.arenas.unions.add(name, fields, span)),
            Tag::Enum => panic!("only for structs and unions"),
        }
    }

    pub fn add_symbol(&mut self, decl: &DeclarationNode) {
        if !decl
            .specifiers
            .contains(&DeclarationSpecifier::Storage(Storage::Typedef))
        {
            return;
        }
        for init_decl in &decl.init_declarators {
            let Some(name) = self.declartor_name(&init_decl.declarator) else {
                continue;
            };
            self.typedefs.insert(name.id);
        }
    }

    pub fn declartor_name(&self, decl: &DeclaratorNode) -> Option<&Name> {
        match self.arenas.declarators.get(decl.id) {
            Declarator::Ident(name) => Some(name),
            Declarator::Pointer { inner: Some(d), .. } => self.declartor_name(d),
            Declarator::Array { declarator, .. } => self.declartor_name(declarator),
            Declarator::Function { declarator, .. } => self.declartor_name(declarator),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ExpressionNode, FunctionParametersNode, InitDeclaratorNode, Tag};

    fn name(s: &str) -> Name {
        let mut arena = StringArena::default();
        arena.add(s.to_string(), Span::default())
    }

    fn typedef_decl(d: DeclaratorNode) -> DeclarationNode {
        DeclarationNode {
            specifiers: vec![
                DeclarationSpecifier::Storage(Storage::Typedef),
                DeclarationSpecifier::Type(TypeSpecifier::Int),
            ],
            init_declarators: vec![InitDeclaratorNode {
                span: Span::default(),
                declarator: d,
                initializer: None,
            }],
            span: Span::default(),
        }
    }

    #[test]
    #[should_panic(expected = "only for structs and unions")]
    fn struct_or_union_rejects_enum_tag() {
        Context::default().struct_or_union(Tag::Enum, None, vec![], Span::default());
    }

    #[test]
    fn add_symbol_stores_typedef_name() {
        let mut ctx = Context::default();
        let ident = name("mytype");
        let d = ctx.arenas.declarators.ident(ident.clone(), Span::default());
        ctx.add_symbol(&typedef_decl(d));
        assert!(ctx.typedefs.contains(&ident.id));
    }

    #[test]
    fn add_symbol_skips_non_typedef() {
        let mut ctx = Context::default();
        let mut decl = typedef_decl(ctx.arenas.declarators.ident(name("x"), Span::default()));
        decl.specifiers = vec![DeclarationSpecifier::Type(TypeSpecifier::Int)];
        ctx.add_symbol(&decl);
        assert!(ctx.typedefs.is_empty());
    }

    #[test]
    fn add_symbol_skips_nameless_declarator() {
        let mut ctx = Context::default();
        let abstract_decl = ctx.arenas.declarators.abstrct(Span::default());
        ctx.add_symbol(&typedef_decl(abstract_decl));
        assert!(ctx.typedefs.is_empty());
    }

    #[test]
    fn declarator_name_recurses_through_array_and_function() {
        let mut ctx = Context::default();
        let ident = ctx
            .arenas
            .declarators
            .ident(ctx.arenas.names.add("arr".into(), Span::default()), Span::default());
        let arr = ctx.arenas.declarators.array(ident, Some(empty_expr()), Span::default());
        let arr_name = ctx.declartor_name(&arr).expect("array declarator name");
        assert_eq!(ctx.arenas.names.get(arr_name.id), "arr");

        let ident2 = ctx
            .arenas
            .declarators
            .ident(ctx.arenas.names.add("fn".into(), Span::default()), Span::default());
        let params = FunctionParametersNode::empty(Span::default());
        let f = ctx.arenas.declarators.function(ident2, params, Span::default());
        assert!(ctx.declartor_name(&f).is_some());
    }

    fn empty_expr() -> ExpressionNode {
        ExpressionNode {
            span: Span::default(),
            id: crate::ast::ExpressionId::from(0),
        }
    }
}
