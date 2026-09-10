use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode};
use crate::codegen::Generator;
use crate::codegen::llvm::{LLVMValue, Op};
use crate::context::Context;
use crate::semantic::sema;

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, ctx: &Context, node: &ExpressionNode) -> LLVMValue {
        match node.id.resolve() {
            Expression::Constant(value_node) => LLVMValue::Literal(value_node.value),
            Expression::Identifier(_) => self.ident(ctx, node),
            Expression::Binary(op, e1, e2) => self.binary(ctx, op, node, e1, e2),
            _ => todo!(),
        }
    }

    fn ident(&mut self, ctx: &Context, node: &ExpressionNode) -> LLVMValue {
        let re = &sema().expr_types[node.id];
        let qty = re.casted_ty();
        let sym_id = sema().expr_bindings[node.id];
        let local = self.locals.get(sym_id);
        let align = qty.layout(ctx).unwrap().align;
        self.b.load(qty.llvm(ctx), local.llvm(ctx), align)
    }

    fn binary(
        &mut self,
        ctx: &Context,
        op: &BinaryOp,
        node: &ExpressionNode,
        e1: &ExpressionNode,
        e2: &ExpressionNode,
    ) -> LLVMValue {
        let sema = sema();
        let re = &sema.expr_types[node.id];
        let f = re.ty.is_floating(sema);
        let op = match (op, f) {
            (BinaryOp::Add, false) => Op::Add,
            (BinaryOp::Sub, false) => Op::Sub,
            (BinaryOp::Mul, false) => Op::Mul,
            (BinaryOp::Div, false) => Op::SDiv,
            (BinaryOp::Mod, false) => Op::SRem,

            (BinaryOp::Add, true) => Op::FAdd,
            (BinaryOp::Sub, true) => Op::FSub,
            _ => todo!(),
        };
        let v1 = self.fold_expression(ctx, e1);
        let v2 = self.fold_expression(ctx, e2);
        let qty = re.casted_ty();
        self.b.binop(op, qty.llvm(ctx), v1, v2)
    }
}
