use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::arena::{Provide, ProvideMut};
use crate::ast::{AstArenas, Name, StringId, StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier};
use crate::parser::{ParseState, Span};
use crate::semantic::{DiagnosisNode, Sema};
use crate::target::Target;

#[derive(Debug, Default)]
pub struct Context {
    pub file_name: String,
    pub target: Target,
    pub diagnosis: Vec<DiagnosisNode>,
    pub parse: ParseState,
    pub arenas: AstArenas,
    pub ast: TranslationUnitNode,
    pub sema: Sema,
    source_cache: RefCell<HashMap<String, Option<Vec<String>>>>,
}

impl Provide<Sema> for Context {
    fn provide(&self) -> &Sema {
        &self.sema
    }
}

impl ProvideMut<Sema> for Context {
    fn provide_mut(&mut self) -> &mut Sema {
        &mut self.sema
    }
}

impl Provide<AstArenas> for Context {
    fn provide(&self) -> &AstArenas {
        &self.arenas
    }
}

impl ProvideMut<AstArenas> for Context {
    fn provide_mut(&mut self) -> &mut AstArenas {
        &mut self.arenas
    }
}

impl Context {
    pub fn set_file_name(&mut self, file_name: String) {
        self.arenas.names.alloc(file_name.clone());
        self.file_name = file_name;
    }

    pub fn file_of(&self, span: Span) -> &String {
        StringId::from(span.start.file).resolve(self)
    }

    pub fn source_line(&self, path: &str, line_no: usize) -> Option<String> {
        let mut cache = self.source_cache.borrow_mut();
        let lines = cache
            .entry(path.to_string())
            .or_insert_with(|| read_to_string(path).ok().map(|text| text.lines().map(str::to_string).collect()));
        lines.as_ref()?.get(line_no.checked_sub(1)?).cloned()
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
