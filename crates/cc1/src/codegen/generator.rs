use std::collections::HashMap;
use std::io::{Write, stdout};

use crate::ast::visit::walk_translation_unit;
use crate::ast::{
    ExpressionNode, FunctionDefinitionNode, JumpStatement, JumpStatementNode, TranslationUnitNode, Visitor,
};
use crate::codegen::AllocaCollector;
use crate::codegen::llvm::Builder;
use crate::context::Context;
use crate::semantic::{ResolvedType, SymbolId};

pub fn generate(ctx: Context) -> Context {
    let mut generator = Generator::new(stdout());
    generator.visit_translation_unit(&ctx, &ctx.ast);
    ctx
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    pub b: Builder<W>,
    pub locals: HashMap<SymbolId, usize>,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self {
            b: Builder::new(w),
            locals: HashMap::new(),
        }
    }

    fn allocas(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        self.locals.clear();
        AllocaCollector::run(&mut self.locals, ctx, node);
        self.b.reset(self.locals.len());
        let mut it: Vec<(SymbolId, usize)> = self
            .locals
            .iter()
            .map(|(&sym_id, &alloc_id)| (sym_id, alloc_id))
            .collect();
        it.sort_by_key(|(_, id)| *id);
        for (sym_id, alloc_id) in it {
            self.alloca(ctx, sym_id, alloc_id);
        }
        self.b.blank();
    }

    fn alloca(&mut self, ctx: &Context, sym_id: SymbolId, id: usize) {
        let qty = sym_id.resolve(ctx).ty.unwrap();
        self.b.alloca(id, qty.llvm(ctx), qty.layout(ctx).unwrap().align);
    }
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode) {
        self.b.target(ctx.target.datalayout, ctx.target.triple);
        self.b.blank();
        walk_translation_unit(self, ctx, node);
    }

    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let sym = ctx.sema.declarations[&node.declarator.id].resolve(ctx);
        let ty = sym.ty.unwrap().id.resolve(ctx);
        let ResolvedType::Function { ret, .. } = ty else { unreachable!() };
        let ret_ty = ret.id.resolve(ctx);
        self.b.define(ret_ty.llvm(ctx), sym.name.id.resolve(ctx));
        self.allocas(ctx, node);
        self.visit_compound_statement(ctx, &node.body);
        self.b.end_function();
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        match &node.stmt {
            JumpStatement::Return(Some(e)) => {
                let ty = ctx.sema.expr_types[e.id].ty.llvm(ctx);
                let v = self.fold_expression(ctx, e);
                self.b.ret(ty, v);
            }
            JumpStatement::Return(None) => self.b.ret_void(),
            _ => todo!(),
        }
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        self.fold_expression(ctx, node);
    }
}
