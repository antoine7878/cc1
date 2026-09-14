use std::io::{Write, stdout};

use libft::Span;

use crate::ast::visit::walk_translation_unit;
use crate::ast::{
    ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode, JumpStatement, JumpStatementNode, TranslationUnitNode,
    Visitor,
};
use crate::codegen::{Builder, Globals, LlvmSymbol, Locals};
use crate::context::ctx;
use crate::semantic::{Definition, Diagnosis, DiagnosisNode, Initializer, SymbolId, sema};

pub fn generate() -> Vec<DiagnosisNode> {
    generate_to(stdout())
}

pub fn generate_to<W: Write>(w: W) -> Vec<DiagnosisNode> {
    let mut generator = Generator::new(w);
    generator.visit_translation_unit(&ctx().ast);
    generator.diagnosis
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    pub b: Builder<W>,
    pub locals: Locals,
    pub globals: Globals,
    pub diagnosis: Vec<DiagnosisNode>,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self { b: Builder::new(w), locals: Locals::default(), globals: Globals::default(), diagnosis: Vec::new() }
    }

    fn emit(&mut self, res: Result<(), Diagnosis>, span: &Span) {
        if let Err(diagnosis) = res {
            self.diagnosis.push(DiagnosisNode::new(diagnosis, *span));
        }
    }

    fn fold_into(&mut self, e: &ExpressionNode, slot: LlvmSymbol) {
        let res = self.fold_expression(e).map(|v| self.b.store(v, slot));
        self.emit(res, &e.span);
    }

    fn emit_locals(&mut self) {
        self.b.reset(self.locals.parameters.len());
        self.locals.emit_decl(&mut self.b);
        for id in self.locals.order_iter() {
            self.emit_init(id);
        }
    }

    fn emit_init(&mut self, id: SymbolId) {
        let sym = id.resolve();
        let Some(init) = sym.initializer else { return };
        let qty = sym.ty;
        match init.resolve() {
            Initializer::Zero => self.b.store(LlvmSymbol::zero(qty), self.locals[id]),
            Initializer::Value(v) => {
                let v = self.constant(qty, *v);
                self.b.store(v, self.locals[id]);
            }
            Initializer::String(s) => {
                let &a = self.globals.get_literal(*s).unwrap();
                self.b.store(a, self.locals[id]);
            }
            Initializer::Expr(e) => self.fold_into(e, self.locals[id]),
            Initializer::List(_) => todo!("list init"),
            Initializer::Address(_) => todo!("address init"),
        }
    }

    fn emit_globals(&mut self) {
        let mut strings: Vec<_> = self.globals.strings.iter().collect();
        strings.sort_by_key(|(id, _)| usize::from(**id));
        for (id, sym) in strings {
            self.b.string_literal(sym.name, id.resolve());
        }
        for index in 0..sema().tags.len() {
            self.b.type_def(index.into());
        }
        for sym in &self.globals.order {
            self.b.global(*sym);
        }
        for sym in &self.globals.functions {
            if sym.resolve().definition != Definition::Definition {
                self.b.declare(*sym);
            }
        }
    }
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, node: &TranslationUnitNode) {
        self.b.target(ctx().target.datalayout, ctx().target.triple);
        self.globals.collect(node);
        self.emit_globals();

        walk_translation_unit(self, node);
    }

    fn visit_function_definition(&mut self, node: &FunctionDefinitionNode) {
        let sym = sema().declarations[&node.declarator.id];
        let f = self.globals.get_symbol(sym).unwrap();
        self.locals.collect(node);
        self.b.define(*f, self.locals.parameters(), self.locals.is_variadic());
        self.emit_locals();
        self.visit_compound_statement(&node.body);
        self.b.end_function();
    }

    fn visit_jump_statement(&mut self, node: &JumpStatementNode) {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => {
                let res = self.fold_expression(e).map(|v| self.b.ret(v));
                self.emit(res, &e.span);
            }
            JumpStatement::Return(None) => self.b.ret_void(),
            _ => todo!(),
        }
    }

    fn visit_expression(&mut self, node: &ExpressionNode) {
        let res = self.fold_expression(node).map(|_| ());
        self.emit(res, &node.span);
    }

    fn visit_init_declarator(&mut self, _node: &InitDeclaratorNode) {}
}
