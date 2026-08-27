use crate::ast::{
    DeclaratorArena, EnumArena, ExpressionArena, Name, StatementArena, StringArena, StringId, StructArena,
    StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, UnionArena, VariantArena,
};
use crate::parser::{ParseState, Span};
use crate::semantic::{DiagnosisNode, Sema};
use crate::target::Target;

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
    pub file_name: String,
    pub target: Target,
    pub diagnosis: Vec<DiagnosisNode>,
    pub parse: ParseState,
    pub arenas: Arenas,
    pub ast: TranslationUnitNode,
    pub sema: Sema,
}

impl Context {
    pub fn set_file_name(&mut self, file_name: String) {
        self.arenas.names.alloc(file_name.clone());
        self.file_name = file_name;
    }

    pub fn file_of(&self, span: Span) -> &String {
        StringId::from(span.start.file).resolve(self)
    }

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
}
