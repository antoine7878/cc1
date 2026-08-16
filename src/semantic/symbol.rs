pub struct SemanticContext {
    scopes: Vec<Scope>,
    diagnostics: Vec<Diagnostic>,
    current_function: Option<FunctionId>,
    loop_depth: usize,
    switch_depth: usize,
}

struct Analyzer<'a> {
    ast: &'a mut TranslationUnit,
    sema: SemanticContext,
}

impl<'a> Analyzer<'a> {
    fn analyze(&mut self) {
        self.collect_declarations();
        self.resolve_names();
        self.type_check();
    }
}
