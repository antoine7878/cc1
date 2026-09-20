use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;

use libft::{SourceMap, Span};

use crate::arena::Global;
use crate::ast::{
    AstArenas, ConstValue, ConstValueNode, ExpressionNode, Name, NameId, StructDeclaration, Tag, TranslationUnitNode,
    TypeSpecifier,
};
use crate::parser::ParseState;
use crate::semantic::DiagnosticNode;

thread_local! {
    static CTX: Global<Context> = const { Global::new("Context") };
}

pub fn install_context(ctx: Context) -> &'static Context {
    CTX.with(|g| g.install(ctx))
}

pub fn ctx() -> &'static Context {
    CTX.with(Global::get)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineMarker {
    pub file: usize,
    pub logical: usize,
    pub physical: usize,
}

#[derive(Debug)]
pub struct Context {
    pub file_name: String,
    pub diagnostics: Vec<DiagnosticNode>,
    pub parse: ParseState,
    pub arenas: AstArenas,
    pub ast: TranslationUnitNode,
    pub line_markers: Vec<LineMarker>,
    source_cache: RefCell<HashMap<String, Option<Vec<String>>>>,
    const_one: ExpressionNode,
}
impl Default for Context {
    fn default() -> Self {
        let mut arenas = AstArenas::default();
        let value_node = ConstValueNode { span: Span::default(), value: ConstValue::Int(1) };
        let const_one = arenas.expressions.constant(value_node, Span::default());
        Self {
            const_one,
            file_name: String::default(),
            diagnostics: Vec::default(),
            parse: ParseState::default(),
            arenas,
            ast: TranslationUnitNode::default(),
            line_markers: Vec::default(),
            source_cache: RefCell::default(),
        }
    }
}

impl Context {
    pub fn const_one(&self) -> ExpressionNode {
        self.const_one.clone()
    }

    pub fn set_file_name(&mut self, file_name: String) {
        let file = self.arenas.names.intern(file_name.clone()).into();
        self.file_name = file_name;
        self.line_markers = vec![LineMarker { file, logical: 1, physical: 1 }];
    }

    pub fn mark_line(&mut self, file: usize, logical: usize, next_line: usize) {
        let physical = match self.line_markers.last() {
            Some(prev) => prev.physical + next_line.saturating_sub(prev.logical),
            None => next_line,
        };
        self.line_markers.push(LineMarker { file, logical, physical });
    }

    fn physical_lines(&self, path: &str, line_no: usize) -> Vec<usize> {
        let mut found = Vec::new();
        for (i, marker) in self.line_markers.iter().enumerate() {
            if self.path_of(marker.file) != Some(path) || marker.logical > line_no {
                continue;
            }
            let physical = marker.physical + (line_no - marker.logical);
            if self.line_markers.get(i + 1).is_none_or(|next| physical < next.physical) {
                found.push(physical);
            }
        }
        found
    }

    fn cached_line(&self, path: &str, line_no: usize) -> Option<String> {
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
        declarations: Vec<StructDeclaration>,
        span: Span,
    ) -> TypeSpecifier {
        match tag {
            Tag::Struct => TypeSpecifier::Struct(self.arenas.structs.add(name, declarations, span)),
            Tag::Union => TypeSpecifier::Union(self.arenas.unions.add(name, declarations, span)),
            Tag::Enum => panic!("only for structs and unions"),
        }
    }
}

impl SourceMap for Context {
    fn path_of(&self, file: usize) -> Option<&str> {
        self.arenas.names.try_get(NameId::from(file)).map(String::as_str)
    }

    fn source_line(&self, path: &str, line_no: usize) -> Option<String> {
        let candidates = self.physical_lines(path, line_no);
        if candidates.is_empty() {
            return self.cached_line(path, line_no);
        }
        candidates.into_iter().find_map(|physical| self.cached_line(&self.file_name, physical))
    }
}
