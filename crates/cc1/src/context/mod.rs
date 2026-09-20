use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs::read_to_string;

use libft::{SourceMap, Span};

use crate::ast::{
    AstArenas, ConstValue, ConstValueNode, ExpressionNode, Name, NameId, StructDeclaration, Tag, TranslationUnitNode,
    TypeSpecifier,
};
use crate::parser::ParseState;
use crate::semantic::DiagnosticNode;

thread_local! {
    static CTX: Cell<Option<&'static Context>> = const { Cell::new(None) };
}

pub fn install_context(ctx: Context) -> &'static Context {
    let ctx = Box::leak(Box::new(ctx));
    CTX.set(Some(ctx));
    ctx
}

pub fn ctx() -> &'static Context {
    CTX.get().expect("Context is not installed")
}

#[derive(Debug)]
pub struct Context {
    pub file_name: String,
    pub diagnostics: Vec<DiagnosticNode>,
    pub parse: ParseState,
    pub arenas: AstArenas,
    pub ast: TranslationUnitNode,
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
            source_cache: RefCell::default(),
        }
    }
}

impl Context {
    pub fn const_one(&self) -> ExpressionNode {
        self.const_one.clone()
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.arenas.names.intern(file_name.clone());
        self.file_name = file_name;
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
        let mut cache = self.source_cache.borrow_mut();
        let lines = cache
            .entry(path.to_string())
            .or_insert_with(|| read_to_string(path).ok().map(|text| text.lines().map(str::to_string).collect()));
        lines.as_ref()?.get(line_no.checked_sub(1)?).cloned()
    }
}
