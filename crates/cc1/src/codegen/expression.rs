use std::cmp::Ordering;
use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode, UnaryOp};
use crate::codegen::llvm::LlvmOperator;
use crate::codegen::{Generator, LlvmName, LlvmSymbol};
use crate::semantic::{ResolvedType, sema};

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, node: &ExpressionNode) -> LlvmSymbol {
        if let Some(value) = sema().expr_consts.get(node.id) {
            return LlvmSymbol::cst(sema().expr_types[node.id].casted_ty().llvm(), *value);
        }
        self.fold_raw(node)
    }

    pub fn fold_raw(&mut self, node: &ExpressionNode) -> LlvmSymbol {
        match node.id.resolve() {
            Expression::Identifier(_) => self.ident(node),
            Expression::StringLiteral(s) => *self.globals.get_literal(s.id).unwrap(),
            Expression::Constant(value_node) => LlvmSymbol::from(value_node.value),
            Expression::Binary(op, lhs, rhs) => self.binary(op, lhs, rhs),
            Expression::Unary(op, e) => self.unary(op, e),
            Expression::Assign(op, lhs, rhs) => self.assign(op, lhs, rhs),
            Expression::List(lst) => lst.iter().fold(LlvmSymbol::from(0), |_, e| self.fold_expression(e)),
            Expression::Ternary(e, lhs, rhs) => self.ternary(e, lhs, rhs),
            Expression::FunctionCall(f, args) => self.call(f, args),
            Expression::ArraySubscripting(array, idx) => self.array_subscript(array, idx),
            Expression::Member(_, _, _) => todo!(),
            Expression::Cast(_, e) => self.cast(node, e),
            Expression::ConstantExpression(_) | Expression::SizeofExpr(_) | Expression::SizeofType(_) => unreachable!(),
        }
    }

    fn ident(&mut self, node: &ExpressionNode) -> LlvmSymbol {
        let re = &sema().expr_types[node.id];
        let sym_id = sema().expr_bindings[node.id];

        if let Some(v) = self.globals.get_function(sym_id.resolve().name.id) {
            return *v;
        }
        let local = self.locals[sym_id];
        self.b.load(re.ty.llvm(), local)
    }

    fn unary(&mut self, op: &UnaryOp, e: &ExpressionNode) -> LlvmSymbol {
        match op {
            UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec => self.unary_inc_dec(op, e),
            UnaryOp::Plus => self.fold_expression(e),
            UnaryOp::Minus => self.unary_minus(e),
            UnaryOp::LogicalNot => self.unary_logic_not(op, e),
            UnaryOp::BitNot => self.unary_bitnot(op, e),
            UnaryOp::Addr => self.fold_expression(e),
            UnaryOp::Deref => self.unary_deref(e),
        }
    }

    fn binary(&mut self, op: &BinaryOp, lhs: &ExpressionNode, rhs: &ExpressionNode) -> LlvmSymbol {
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
            | BinaryOp::BitXor => self.binary_arithmetic(op, lhs, rhs),
            BinaryOp::Greater
            | BinaryOp::Lower
            | BinaryOp::GreaterEq
            | BinaryOp::LowerEq
            | BinaryOp::Eq
            | BinaryOp::Neq => self.binary_comparison(op, lhs, rhs),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => self.binary_logical(op, lhs, rhs),
        }
    }

    fn unary_inc_dec(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> LlvmSymbol {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let sym_id = sema().expr_bindings.get(operand.id).unwrap();
        let loc = *self.locals.get(*sym_id).unwrap();
        let vr = self.fold_expression(operand);
        let lop = LlvmOperator::unary(op, qty);
        let v = self.b.binop(lop, vr, LlvmSymbol::from(1));
        self.b.store(v, loc);
        match op {
            UnaryOp::PreDec | UnaryOp::PreInc => v,
            UnaryOp::PostDec | UnaryOp::PostInc => vr,
            _ => unreachable!(),
        }
    }

    fn unary_minus(&mut self, operand: &ExpressionNode) -> LlvmSymbol {
        let v = self.fold_expression(operand);
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let op = LlvmOperator::binary(&BinaryOp::Sub, qty);
        self.b.binop(op, LlvmSymbol::from(0), v)
    }

    fn unary_logic_not(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> LlvmSymbol {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.fold_expression(operand);
        let lop = LlvmOperator::binary(&BinaryOp::Neq, qty);
        let v = self.b.binop(lop, v, LlvmSymbol::from(0));
        let xor = LlvmOperator::unary(op, qty);
        self.b.binop(xor, v, LlvmSymbol::from(true))
    }

    fn unary_bitnot(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> LlvmSymbol {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.fold_expression(operand);
        let xor = LlvmOperator::unary(op, qty);
        self.b.binop(xor, v, LlvmSymbol::from(-1))
    }

    fn unary_deref(&mut self, operand: &ExpressionNode) -> LlvmSymbol {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.fold_expression(operand);
        let ResolvedType::Pointer(qty) = qty.id.resolve() else { unreachable!() };
        self.b.load(qty.llvm(), v)
    }

    fn binary_arithmetic(&mut self, op: &BinaryOp, lhs: &ExpressionNode, rhs: &ExpressionNode) -> LlvmSymbol {
        let sema = sema();
        let rel = &sema.expr_types[lhs.id];
        let qty = rel.casted_ty();
        let op = LlvmOperator::binary(op, qty);
        let v1 = self.fold_expression(lhs);
        let v2 = self.fold_expression(rhs);
        self.b.binop(op, v1, v2)
    }

    fn binary_comparison(&mut self, op: &BinaryOp, lhs: &ExpressionNode, rhs: &ExpressionNode) -> LlvmSymbol {
        let rel = &sema().expr_types[lhs.id];
        let op = LlvmOperator::binary(op, rel.casted_ty());
        let v1 = self.fold_expression(lhs);
        let v2 = self.fold_expression(rhs);
        self.b.binop(op, v1, v2)
        // self.b.zext(LlvmType::bool(), v, re.ty.llvm())
    }

    fn binary_logical(&mut self, op: &BinaryOp, lhs: &ExpressionNode, rhs: &ExpressionNode) -> LlvmSymbol {
        let initial_block = self.b.current_block;
        let rel = &sema().expr_types[lhs.id];
        let cmp = LlvmOperator::binary(op, rel.ty);

        let v = self.fold_expression(lhs);
        let v = self.b.binop(cmp, v, LlvmSymbol::from(0));
        let LlvmName::SSA(i) = v.name else { unreachable!() };
        let l1 = LlvmName::label(i, 1);
        let l2 = LlvmName::label(i, 2);
        match op {
            BinaryOp::LogicalAnd => self.b.br(v, l1, Some(l2)),
            BinaryOp::LogicalOr => self.b.br(v, l2, Some(l1)),
            _ => unreachable!(),
        }

        self.b.named_label(l1);
        let v = self.fold_expression(rhs);
        let v = self.b.binop(cmp, v, LlvmSymbol::from(0));
        self.b.br(v, l2, None);

        self.b.named_label(l2);
        self.b.phi((*op == BinaryOp::LogicalOr).into(), initial_block, v, l1)
    }

    fn assign(&mut self, op: &Option<BinaryOp>, lhs: &ExpressionNode, rhs: &ExpressionNode) -> LlvmSymbol {
        let rl = &sema().expr_types[lhs.id];
        let qty = rl.casted_ty();
        let sym_id = sema().expr_bindings.get(lhs.id).unwrap();
        let loc = *self.locals.get(*sym_id).unwrap();
        let mut vr = self.fold_expression(rhs);
        if let Some(op) = op {
            let vl = self.b.load(rl.casted_ty().llvm(), loc);
            let op = LlvmOperator::binary(op, qty);
            vr = self.b.binop(op, vl, vr);
        }
        self.b.store(vr, loc);
        vr
    }

    fn ternary(&mut self, cond: &ExpressionNode, a: &ExpressionNode, b: &ExpressionNode) -> LlvmSymbol {
        let v = self.fold_expression(cond);
        let v = self.b.binop("icmp ne", v, LlvmSymbol::from(0));

        let LlvmName::SSA(i) = v.name else { unreachable!() };
        let l1 = LlvmName::label(i, 0);
        let l2 = LlvmName::label(i, 1);
        let l3 = LlvmName::label(i, 2);
        self.b.br(v, l1, Some(l2));

        self.b.named_label(l1);
        let va = self.fold_expression(a);
        self.b.br(v, l3, None);

        self.b.named_label(l2);
        let vb = self.fold_expression(b);
        self.b.br(v, l3, None);

        self.b.named_label(l3);
        self.b.phi(va, l1, vb, l2)
    }

    fn call(&mut self, f: &ExpressionNode, _args: &[ExpressionNode]) -> LlvmSymbol {
        let f = self.fold_expression(f);
        self.b.call(f)
    }

    fn array_subscript(&mut self, array: &ExpressionNode, idx: &ExpressionNode) -> LlvmSymbol {
        let arr = self.fold_expression(array);
        let idx1 = LlvmSymbol::idx(0);
        let i = sema().expr_consts[idx.id].get_integer_value().unwrap();
        let idx2 = LlvmSymbol::idx(i);

        let v = self.b.getelementptr(arr, idx1, idx2);
        let ty = sema().expr_types[array.id].ty.llvm();
        self.b.load(ty, v)
    }

    fn cast(&mut self, node: &ExpressionNode, operand: &ExpressionNode) -> LlvmSymbol {
        let from_ty = sema().expr_types[operand.id].casted_ty();
        let from_size = sema().layout(&from_ty.id).size;

        let to_ty = sema().expr_types[node.id].ty;
        let to_size = sema().layout(&to_ty.id).size;
        let v = self.fold_expression(operand);

        match u32::cmp(&from_size, &to_size) {
            Ordering::Less => self.b.zext(v, to_ty.llvm()),
            Ordering::Greater => self.b.trunc(v, to_ty.llvm()),
            Ordering::Equal => v,
        }
    }
}
