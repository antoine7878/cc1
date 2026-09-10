use std::io::{Write, stdout};

use crate::ast::visit::walk_translation_unit;
use crate::ast::{
    ExpressionNode, FunctionDefinitionNode, JumpStatement, JumpStatementNode, TranslationUnitNode, Visitor,
};
use crate::codegen::llvm::Builder;
use crate::codegen::local::Locals;
use crate::context::Context;
use crate::semantic::ResolvedType;

pub fn generate(ctx: Context) -> Context {
    let mut generator = Generator::new(stdout());
    generator.visit_translation_unit(&ctx, &ctx.ast);
    ctx
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    pub b: Builder<W>,
    pub locals: Locals,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self {
            b: Builder::new(w),
            locals: Locals::default(),
        }
    }

    fn allocas(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        self.locals.collect(ctx, node);
        self.b.reset(0);
        self.locals.emit(ctx, &mut self.b);
        self.b.blank();
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
