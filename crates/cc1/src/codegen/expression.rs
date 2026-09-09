use std::fmt;
use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode, Value};
use crate::codegen::Generator;
use crate::context::Context;
use crate::emitln;

#[derive(Debug)]
pub enum LLVMValue {
    SSA(usize),
    Literal(Value),
    // Global(NameId),
}

impl fmt::Display for LLVMValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLVMValue::SSA(id) => write!(f, "%{id}"),
            LLVMValue::Literal(value) => write!(f, "{value}"),
        }
    }
}

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, ctx: &Context, node: &ExpressionNode) -> LLVMValue {
        match node.id.resolve(ctx) {
            Expression::Constant(value_node) => LLVMValue::Literal(value_node.value),
            Expression::Identifier(_) => self.ident(ctx, node),
            Expression::Binary(op, e1, e2) => self.binary(ctx, op, node, e1, e2),
            _ => todo!(),
        }
    }

    fn ident(&mut self, ctx: &Context, node: &ExpressionNode) -> LLVMValue {
        let re = &ctx.sema.expr_types[node.id];
        let qty = re.casted_ty();
        let sym_id = ctx.sema.expr_bindings[node.id];
        let r = self.next_id();
        let ty = qty.llvm(ctx);
        let slot = self.locals[&sym_id];
        let align = qty.layout(ctx).unwrap().align;
        emitln!(self, "  %{r} = load {ty}, ptr %{slot}, align {align}");
        LLVMValue::SSA(r)
    }

    fn binary(
        &mut self,
        ctx: &Context,
        op: &BinaryOp,
        node: &ExpressionNode,
        e1: &ExpressionNode,
        e2: &ExpressionNode,
    ) -> LLVMValue {
        let re = &ctx.sema.expr_types[node.id];
        let f = re.ty.is_floating(&ctx.sema);
        let op = match (op, f) {
            (BinaryOp::Add, false) => "add nsw",
            (BinaryOp::Sub, false) => "sub nsw",
            (BinaryOp::Mul, false) => "mul nsw",
            (BinaryOp::Div, false) => "sdiv",
            (BinaryOp::Mod, false) => "srem",

            (BinaryOp::Add, true) => "fadd",
            (BinaryOp::Sub, true) => "fsub",
            _ => todo!(),
        };
        let v1 = self.fold_expression(ctx, e1);
        let v2 = self.fold_expression(ctx, e2);
        let qty = re.casted_ty();
        let r = self.next_id();
        let ty = qty.llvm(ctx);
        emitln!(self, "  %{r} = {op} {ty} {v1}, {v2}");
        LLVMValue::SSA(r)
    }
}
