use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode};
use crate::codegen::Generator;
use crate::codegen::llvm::{LlvmOperator, LlvmValue};
use crate::semantic::sema;

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, node: &ExpressionNode) -> LlvmValue {
        match node.id.resolve() {
            Expression::Constant(value_node) => LlvmValue::Literal(value_node.value),
            Expression::Identifier(_) => self.ident(node),
            Expression::Binary(op, e1, e2) if op.is_arithmetic() => self.binary_arithmetic(op, node, e1, e2),
            Expression::Binary(op, e1, e2) if op.is_comparison() => self.binary_comparison(op, node, e1, e2),
            Expression::Binary(op, e1, e2) if op.is_logical() => todo!(),
            _ => todo!(),
        }
    }

    fn ident(&mut self, node: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[node.id];
        let qty = re.casted_ty();
        let sym_id = sema().expr_bindings[node.id];
        let local = self.locals.get(sym_id);
        let align = qty.layout().unwrap().align;
        self.b.load(qty.llvm(), local, align)
    }

    fn binary_arithmetic(
        &mut self,
        op: &BinaryOp,
        _node: &ExpressionNode,
        e1: &ExpressionNode,
        e2: &ExpressionNode,
    ) -> LlvmValue {
        let sema = sema();
        let re1 = &sema.expr_types[e1.id];
        let qty = re1.casted_ty();
        let op = LlvmOperator::get(qty, op);
        let v1 = self.fold_expression(e1);
        let v2 = self.fold_expression(e2);
        self.b.binop(op, qty.llvm(), v1, v2)
    }

    fn binary_comparison(
        &mut self,
        op: &BinaryOp,
        node: &ExpressionNode,
        e1: &ExpressionNode,
        e2: &ExpressionNode,
    ) -> LlvmValue {
        let sema = sema();
        let re = &sema.expr_types[node.id];
        let re1 = &sema.expr_types[e1.id];
        let qty = re1.ty;
        let op = LlvmOperator::get(qty, op);
        let v1 = self.fold_expression(e1);
        let v2 = self.fold_expression(e2);
        let v = self.b.binop(op, qty.llvm(), v1, v2);
        self.b.zext_bool(v, re.casted_ty().llvm())
    }
}
