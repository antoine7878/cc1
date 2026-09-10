use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode, UnaryOp};
use crate::codegen::Generator;
use crate::codegen::llvm::{LlvmOperator, LlvmValue};
use crate::semantic::{ResolvedType, sema};

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, node: &ExpressionNode) -> LlvmValue {
        match node.id.resolve() {
            Expression::Constant(value_node) => LlvmValue::Constant(value_node.value),
            Expression::Identifier(_) => self.ident(node),
            Expression::StringLiteral(s) => self.globals.get(s.id),
            Expression::ConstantExpression(e) => LlvmValue::Constant(sema().expr_consts[e.id]),
            Expression::Binary(op, e1, e2) => self.binary(op, node, e1, e2),
            Expression::Unary(op, e) => self.unary(op, node, e),
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

    fn unary(&mut self, op: &UnaryOp, node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        use UnaryOp::*;
        match op {
            PostInc | PostDec | PreInc | PreDec => self.unary_inc_dec(op, node, e),
            Plus => self.fold_expression(e),
            Minus => self.unary_minus(node, e),
            LogicalNot => self.unary_logic_not(op, node, e),
            BitNot => self.unary_bitnot(op, node, e),
            Addr => self.fold_expression(e),
            Deref => self.unary_deref(node, e),
        }
    }

    fn unary_minus(&mut self, _node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        let v = self.fold_expression(e);
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let op = LlvmOperator::binary(&BinaryOp::Sub, qty);
        self.b.binop(op, qty.llvm(), LlvmValue::zero_cst(), v)
    }

    fn unary_logic_not(&mut self, op: &UnaryOp, _node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let v = self.fold_expression(e);
        let lop = LlvmOperator::binary(&BinaryOp::LogicalAnd, qty);
        let v = self.b.binop(lop, qty.llvm(), v, LlvmValue::zero_cst());
        let op = LlvmOperator::unary(op, qty);
        let v = self.b.binop(op, qty.llvm(), v, true.into());
        self.b.zext_bool(v, re.casted_ty().llvm())
    }

    fn unary_bitnot(&mut self, op: &UnaryOp, _node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let v = self.fold_expression(e);
        let op = LlvmOperator::unary(op, qty);
        self.b.binop(op, qty.llvm(), v, LlvmValue::minus_one_cst())
    }

    fn unary_inc_dec(&mut self, op: &UnaryOp, _node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let sym_id = sema().expr_bindings.get(e.id).unwrap();
        let loc = self.locals.get(*sym_id);
        let vr = self.fold_expression(e);
        let lop = LlvmOperator::unary(op, qty);
        let v = self.b.binop(lop, qty.llvm(), vr, LlvmValue::one_cst());
        let align = qty.layout().unwrap().align;
        self.b.store(qty.llvm(), v, loc, align);
        match op {
            UnaryOp::PreDec | UnaryOp::PreInc => v,
            UnaryOp::PostDec | UnaryOp::PostInc => vr,
            _ => unreachable!(),
        }
    }

    fn unary_deref(&mut self, _node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let v = self.fold_expression(e);
        let ResolvedType::Pointer(qty) = qty.id.resolve() else { unreachable!() };
        let align = qty.layout().unwrap().align;
        self.b.load(qty.llvm(), v, align)
    }

    fn binary(&mut self, op: &BinaryOp, node: &ExpressionNode, e1: &ExpressionNode, e2: &ExpressionNode) -> LlvmValue {
        use BinaryOp::*;
        match op {
            Add | Sub | Mul | Div | Mod | Left | Right | BitAnd | BitOr | BitXor => {
                self.binary_arithmetic(op, node, e1, e2)
            }
            Greater | Lower | GreaterEq | LowerEq | Eq | Neq => self.binary_comparison(op, node, e1, e2),
            LogicalAnd | LogicalOr => self.binary_logical(op, node, e1, e2),
        }
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
        let op = LlvmOperator::binary(op, qty);
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
        let re = &sema().expr_types[node.id];
        let re1 = &sema().expr_types[e1.id];
        let qty = re1.ty;
        let op = LlvmOperator::binary(op, qty);
        let v1 = self.fold_expression(e1);
        let v2 = self.fold_expression(e2);
        let v = self.b.binop(op, qty.llvm(), v1, v2);
        self.b.zext_bool(v, re.casted_ty().llvm())
    }

    fn binary_logical(
        &mut self,
        op: &BinaryOp,
        node: &ExpressionNode,
        e1: &ExpressionNode,
        e2: &ExpressionNode,
    ) -> LlvmValue {
        let re = &sema().expr_types[node.id];
        let re1 = &sema().expr_types[e1.id];
        let re2 = &sema().expr_types[e2.id];
        let initial_block = self.b.current_block;
        let llvm_op = LlvmOperator::binary(op, re.ty);

        let v = self.fold_expression(e1);
        let v = self.b.binop(llvm_op, re1.ty.llvm(), v, LlvmValue::zero_cst());
        let LlvmValue::SSA(i) = v else { unreachable!() };
        let (l1, l2) = LlvmValue::fork(i);
        match op {
            BinaryOp::LogicalAnd => self.b.br(v, l1, Some(l2)),
            BinaryOp::LogicalOr => self.b.br(v, l2, Some(l1)),
            _ => unreachable!(),
        }

        self.b.named_label(l1);
        let v = self.fold_expression(e2);
        let v = self.b.binop(llvm_op, re2.ty.llvm(), v, LlvmValue::zero_cst());
        self.b.br(v, l2, None);

        self.b.named_label(l2);
        let v = self.b.phi((*op == BinaryOp::LogicalOr).into(), initial_block, v, l1);
        self.b.zext_bool(v, re.casted_ty().llvm())
    }
}
