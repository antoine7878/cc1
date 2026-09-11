use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode, UnaryOp};
use crate::codegen::llvm::{LlvmOperator, LlvmValue};
use crate::codegen::{Generator, LlvmType};
use crate::semantic::{ResolvedType, sema};

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, node: &ExpressionNode) -> LlvmValue {
        match node.id.resolve() {
            Expression::Constant(value_node) => LlvmValue::Constant(value_node.value),
            Expression::Identifier(_) => self.ident(node),
            Expression::StringLiteral(s) => *self.globals.get_literal(s.id).unwrap(),
            Expression::ConstantExpression(e) => LlvmValue::Constant(sema().expr_consts[e.id]),
            Expression::Binary(op, lhs, rhs) => self.binary(node, op, lhs, rhs),
            Expression::Unary(op, e) => self.unary(node, op, e),
            Expression::Assign(op, lhs, rhs) => self.assign(node, op, lhs, rhs),
            Expression::List(lst) => lst.iter().fold(LlvmValue::zero(), |_, e| self.fold_expression(e)),
            Expression::Ternary(e, lhs, rhs) => self.ternary(node, e, lhs, rhs),
            Expression::ArraySubscripting { .. } => todo!(),
            Expression::FunctionCall(f, args) => self.call(f, args),
            Expression::Member(_, _, _) => todo!(),
            Expression::SizeofExpr(_) => todo!(),
            Expression::SizeofType(_) => todo!(),
            Expression::Cast(_, _) => todo!(),
        }
    }

    fn ident(&mut self, node: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[node.id];
        let qty = re.casted_ty();
        let sym_id = sema().expr_bindings[node.id];

        if let Some(v) = self.globals.get_function(sym_id.resolve().name.id) {
            return *v;
        }
        let local = self.locals.get(sym_id).unwrap();
        self.b.load(qty.llvm(), *local, qty.align())
    }

    fn unary(&mut self, node: &ExpressionNode, op: &UnaryOp, e: &ExpressionNode) -> LlvmValue {
        match op {
            UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec => self.unary_inc_dec(node, op, e),
            UnaryOp::Plus => self.fold_expression(e),
            UnaryOp::Minus => self.unary_minus(node, e),
            UnaryOp::LogicalNot => self.unary_logic_not(node, op, e),
            UnaryOp::BitNot => self.unary_bitnot(node, op, e),
            UnaryOp::Addr => self.fold_expression(e),
            UnaryOp::Deref => self.unary_deref(node, e),
        }
    }

    fn unary_minus(&mut self, _node: &ExpressionNode, e: &ExpressionNode) -> LlvmValue {
        let v = self.fold_expression(e);
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let op = LlvmOperator::binary(&BinaryOp::Sub, qty);
        self.b.binop(op, qty.llvm(), LlvmValue::zero(), v)
    }

    fn unary_logic_not(&mut self, _node: &ExpressionNode, op: &UnaryOp, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let v = self.fold_expression(e);
        let lop = LlvmOperator::binary(&BinaryOp::LogicalAnd, qty);
        let v = self.b.binop(lop, qty.llvm(), v, LlvmValue::zero());
        let op = LlvmOperator::unary(op, qty);
        let v = self.b.binop(op, qty.llvm(), v, true.into());
        self.b.zext_bool(v, re.casted_ty().llvm())
    }

    fn unary_bitnot(&mut self, _node: &ExpressionNode, op: &UnaryOp, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let v = self.fold_expression(e);
        let op = LlvmOperator::unary(op, qty);
        self.b.binop(op, qty.llvm(), v, LlvmValue::minus_one())
    }

    fn unary_inc_dec(&mut self, _node: &ExpressionNode, op: &UnaryOp, e: &ExpressionNode) -> LlvmValue {
        let re = &sema().expr_types[e.id];
        let qty = re.casted_ty();
        let sym_id = sema().expr_bindings.get(e.id).unwrap();
        let loc = *self.locals.get(*sym_id).unwrap();
        let vr = self.fold_expression(e);
        let lop = LlvmOperator::unary(op, qty);
        let v = self.b.binop(lop, qty.llvm(), vr, LlvmValue::one());
        self.b.store(qty.llvm(), v, loc, qty.align());
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
        self.b.load(qty.llvm(), v, qty.align())
    }

    fn binary(
        &mut self,
        node: &ExpressionNode,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> LlvmValue {
        match op {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Left
            | BinaryOp::Right
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor => self.binary_arithmetic(node, op, lhs, rhs),
            BinaryOp::Greater
            | BinaryOp::Lower
            | BinaryOp::GreaterEq
            | BinaryOp::LowerEq
            | BinaryOp::Eq
            | BinaryOp::Neq => self.binary_comparison(node, op, lhs, rhs),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => self.binary_logical(node, op, lhs, rhs),
        }
    }

    fn binary_arithmetic(
        &mut self,
        _node: &ExpressionNode,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> LlvmValue {
        let sema = sema();
        let rel = &sema.expr_types[lhs.id];
        let qty = rel.casted_ty();
        let op = LlvmOperator::binary(op, qty);
        let v1 = self.fold_expression(lhs);
        let v2 = self.fold_expression(rhs);
        self.b.binop(op, qty.llvm(), v1, v2)
    }

    fn binary_comparison(
        &mut self,
        node: &ExpressionNode,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> LlvmValue {
        let re = &sema().expr_types[node.id];
        let rel = &sema().expr_types[lhs.id];
        let qty = rel.ty;
        let op = LlvmOperator::binary(op, qty);
        let v1 = self.fold_expression(lhs);
        let v2 = self.fold_expression(rhs);
        let v = self.b.binop(op, qty.llvm(), v1, v2);
        self.b.zext_bool(v, re.casted_ty().llvm())
    }

    fn binary_logical(
        &mut self,
        node: &ExpressionNode,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> LlvmValue {
        let re = &sema().expr_types[node.id];
        let rel = &sema().expr_types[lhs.id];
        let rer = &sema().expr_types[rhs.id];
        let initial_block = self.b.current_block;
        let llvm_op = LlvmOperator::binary(op, re.ty);

        let v = self.fold_expression(lhs);
        let v = self.b.binop(llvm_op, rel.ty.llvm(), v, LlvmValue::zero());
        let LlvmValue::SSA(i) = v else { unreachable!() };
        let l1 = LlvmValue::label(i, 1);
        let l2 = LlvmValue::label(i, 2);
        match op {
            BinaryOp::LogicalAnd => self.b.br(v, l1, Some(l2)),
            BinaryOp::LogicalOr => self.b.br(v, l2, Some(l1)),
            _ => unreachable!(),
        }

        self.b.named_label(l1);
        let v = self.fold_expression(rhs);
        let v = self.b.binop(llvm_op, rer.ty.llvm(), v, LlvmValue::zero());
        self.b.br(v, l2, None);

        self.b.named_label(l2);
        let v = self.b.phi(LlvmType::bool(), (*op == BinaryOp::LogicalOr).into(), initial_block, v, l1);
        self.b.zext_bool(v, re.casted_ty().llvm())
    }

    fn assign(
        &mut self,
        _node: &ExpressionNode,
        op: &Option<BinaryOp>,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> LlvmValue {
        let rl = &sema().expr_types[lhs.id];
        let qty = rl.casted_ty();
        let sym_id = sema().expr_bindings.get(lhs.id).unwrap();
        let loc = *self.locals.get(*sym_id).unwrap();
        let mut vr = self.fold_expression(rhs);
        if let Some(op) = op {
            let vl = self.b.load(qty.llvm(), loc, qty.align());
            let op = LlvmOperator::binary(op, qty);
            vr = self.b.binop(op, qty.llvm(), vl, vr);
        }
        self.b.store(qty.llvm(), vr, loc, qty.align());
        vr
    }

    fn ternary(
        &mut self,
        node: &ExpressionNode,
        e: &ExpressionNode,
        a: &ExpressionNode,
        b: &ExpressionNode,
    ) -> LlvmValue {
        let re = &sema().expr_types[node.id];
        let ree = &sema().expr_types[e.id];
        let qty = ree.casted_ty();
        let v = self.fold_expression(e);
        let v = self.b.binop("icmp ne", qty.llvm(), v, LlvmValue::zero());

        let LlvmValue::SSA(i) = v else { unreachable!() };
        let l1 = LlvmValue::label(i, 6);
        let l2 = LlvmValue::label(i, 8);
        let l3 = LlvmValue::label(i, 10);
        self.b.br(v, l1, Some(l2));

        self.b.named_label(l1);
        let va = self.fold_expression(a);
        self.b.br(v, l3, None);

        self.b.named_label(l2);
        let vb = self.fold_expression(b);
        self.b.br(v, l3, None);

        self.b.named_label(l3);
        self.b.phi(re.casted_ty().llvm(), va, l1, vb, l2)
    }

    fn call(&mut self, f: &ExpressionNode, _args: &[ExpressionNode]) -> LlvmValue {
        let re = &sema().expr_types[f.id];
        let qty = re.casted_ty();
        let f = self.fold_expression(f);
        // let s = sema().expr_bindings[f.id].resolve();
        // let g = self.globals.get_function(s.name.id);
        self.b.call(qty.llvm(), f);
        LlvmValue::zero()
    }
}
