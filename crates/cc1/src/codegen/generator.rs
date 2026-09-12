use std::io::{Write, stdout};

use crate::ast::visit::walk_translation_unit;
use crate::ast::{
    ExpressionNode, FunctionDefinitionNode, JumpStatement, JumpStatementNode, TranslationUnitNode, Visitor,
};
use crate::codegen::{Builder, Globals, LlvmSymbol, Locals};
use crate::context::ctx;
use crate::semantic::{Initializer, sema};

pub fn generate() {
    generate_to(stdout());
}

pub fn generate_to<W: Write>(w: W) {
    Generator::new(w).visit_translation_unit(&ctx().ast);
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    pub b: Builder<W>,
    pub locals: Locals,
    pub globals: Globals,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self { b: Builder::new(w), locals: Locals::default(), globals: Globals::default() }
    }

    fn allocas(&mut self, node: &FunctionDefinitionNode) {
        self.locals.collect(node);
        self.b.reset(0);
        self.locals.emit(&mut self.b);

        for id in self.locals.order_iter() {
            let sym = id.resolve();
            let Some(init) = sym.initializer else { continue };
            let ty = id.resolve().ty.llvm();
            match init.resolve() {
                Initializer::Zero => self.b.store(LlvmSymbol::from(0), self.locals[id]),
                Initializer::Value(v) => self.b.store(LlvmSymbol::cst(ty, *v), self.locals[id]),
                Initializer::String(s) => {
                    let &a = self.globals.get_literal(*s).unwrap();
                    self.b.store(a, self.locals[id]);
                }
                Initializer::List(_) => todo!("list init"),
                Initializer::Expr(e) => {
                    let v = self.fold_expression(e);
                    self.b.store(v, self.locals[id])
                }
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
        self.b.define(*f);
        self.allocas(node);
        self.visit_compound_statement(&node.body);
        self.b.end_function();
    }

    fn visit_jump_statement(&mut self, node: &JumpStatementNode) {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => {
                let v = self.fold_expression(e);
                self.b.ret(v);
            }
            JumpStatement::Return(None) => self.b.ret_void(),
            _ => todo!(),
        }
    }

    fn visit_expression(&mut self, node: &ExpressionNode) {
        self.fold_expression(node);
    }
}
