use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::ast::{
    AstArenas, ExpressionNode, Name, StringId, StructDeclaration, Tag, TranslationUnitNode, TypeSpecifier, Value,
    ValueNode,
};
use crate::parser::ParseState;
use crate::semantic::{DiagnosisNode, Sema};
use crate::target::Target;
use libft::{SourceMap, Span};

#[derive(Debug)]
pub struct Context {
    pub file_name: String,
    pub target: Target,
    pub diagnosis: Vec<DiagnosisNode>,
    pub parse: ParseState,
    pub arenas: AstArenas,
    pub ast: TranslationUnitNode,
    pub sema: Sema,
    source_cache: RefCell<HashMap<String, Option<Vec<String>>>>,
    one: ExpressionNode,
}
impl Default for Context {
    fn default() -> Self {
        Self::with_target(Target::default())
    }
}

impl Context {
    pub fn with_target(target: Target) -> Self {
        let mut arenas = AstArenas::default();
        let value_node = ValueNode {
            span: Span::default(),
            value: Value::Int(1),
        };
        let one = arenas.expressions.constant(value_node, Span::default());
        Self {
            one,
            file_name: String::default(),
            sema: Sema::new(target.clone()),
            target,
            diagnosis: Vec::default(),
            parse: ParseState::default(),
            arenas,
            ast: TranslationUnitNode::default(),
            source_cache: RefCell::default(),
        }
    }

    pub fn one(&self) -> ExpressionNode {
        self.one.clone()
    }

    pub fn set_target(&mut self, target: Target) {
        self.sema = Sema::new(target.clone());
        self.target = target;
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.arenas.names.alloc(file_name.clone());
        self.file_name = file_name;
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

impl SourceMap for Context {
    fn path_of(&self, file: usize) -> Option<&str> {
        self.arenas.names.try_get(StringId::from(file)).map(String::as_str)
    }

    fn source_line(&self, path: &str, line_no: usize) -> Option<String> {
        let mut cache = self.source_cache.borrow_mut();
        let lines = cache.entry(path.to_string()).or_insert_with(|| {
            read_to_string(path)
                .ok()
                .map(|text| text.lines().map(str::to_string).collect())
        });
        lines.as_ref()?.get(line_no.checked_sub(1)?).cloned()
    }
}
