use std::io::{Write, stdout};

use libft::Span;

use crate::ast::visit::walk_translation_unit;
use crate::ast::{
    ExpressionNode, FunctionDefinitionNode, JumpStatement, JumpStatementNode, TranslationUnitNode, Visitor,
};
use crate::codegen::{Builder, Globals, LlvmSymbol, Locals};
use crate::context::ctx;
use crate::semantic::{Diagnosis, DiagnosisNode, Initializer, sema};

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

    fn allocas(&mut self) {
        self.b.reset(self.locals.parameters.len().saturating_sub(1));
        self.locals.emit(&mut self.b);

        for id in self.locals.order_iter() {
            let sym = id.resolve();
            let Some(init) = sym.initializer else { continue };
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
    }
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, node: &TranslationUnitNode) {
        self.b.target(ctx().target.datalayout, ctx().target.triple);
        self.globals.emit(&mut self.b);
        walk_translation_unit(self, node);
    }

    fn visit_function_definition(&mut self, node: &FunctionDefinitionNode) {
        let sym = sema().declarations[&node.declarator.id].resolve();
        let f = self.globals.get_function(sym.name.id).unwrap();
        self.locals.collect(node);
        self.b.define(*f, self.locals.parameters.as_slice());
        self.allocas();
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
}
