use std::cmp::Ordering;
use std::io::Write;

use crate::ast::{BinaryOp, Expression, ExpressionNode, UnaryOp};
use crate::codegen::{Generator, Invariant, LlvmName, LlvmOperator, LlvmSymbol, LlvmType};
use crate::context::ctx;
use crate::semantic::{CastKind, Diagnosis, ImplicitCast, QualifiedType, sema};

impl<W: Write> Generator<W> {
    pub fn fold_expression(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        if let Some(value) = sema().expr_consts.get(node.id) {
            return Ok(LlvmSymbol::cst(sema().expr_types[node.id].casted_ty().llvm(), *value));
        }
        let s = self.fold_raw(node)?;
        self.apply_casts(s, node)
    }

    fn apply_casts(&mut self, mut s: LlvmSymbol, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let re = &sema().expr_types[node.id];
        let mut from = re.ty;
        for cast in &re.casts {
            s = self.convert(s, from, cast)?;
            from = cast.to;
        }
        Ok(s)
    }

    fn convert(&mut self, v: LlvmSymbol, from: QualifiedType, cast: &ImplicitCast) -> Result<LlvmSymbol, Diagnosis> {
        let mut v = v;
        let to = cast.to;
        let conv = match cast.kind {
            CastKind::ArrayToPointer | CastKind::FunctionToPointer | CastKind::PointerConversion => return Ok(v),
            CastKind::LValueToRValue => return Ok(self.b.load(to.llvm(), v)),
            CastKind::ToVoid => return Ok(LlvmSymbol::void()),
            CastKind::NullPointer => return Ok(LlvmSymbol::null()),
            CastKind::IntegerPromotion | CastKind::IntegerConversion => Self::i_to_i(from, to),
            CastKind::IntegerToFloating => Some(Self::i_to_f(from)),
            CastKind::FloatingToInteger => Some(Self::f_to_i(to)),
            CastKind::FloatingConversion => Self::f_to_f(from, to),
            CastKind::PointerToInteger => Some("ptrtoint"),
            CastKind::IntegerToPointer => self.i_to_p(&mut v, from),
        };
        Ok(match conv {
            Some(conv) => self.b.convert(conv, v, to.llvm()),
            None => v,
        })
    }

    fn i_to_i(from: QualifiedType, to: QualifiedType) -> Option<&'static str> {
        Self::i_to_i_size(from, sema().layout(&to.id).size)
    }

    fn i_to_i_size(from: QualifiedType, to_size: u32) -> Option<&'static str> {
        match u32::cmp(&sema().layout(&from.id).size, &to_size) {
            Ordering::Less if from.is_signed(sema()) => Some("sext"),
            Ordering::Less => Some("zext"),
            Ordering::Greater => Some("trunc"),
            Ordering::Equal => None,
        }
    }

    fn i_to_f(from: QualifiedType) -> &'static str {
        if from.is_signed(sema()) { "sitofp" } else { "uitofp" }
    }

    fn f_to_i(to: QualifiedType) -> &'static str {
        if to.is_signed(sema()) { "fptosi" } else { "fptoui" }
    }

    fn f_to_f(from: QualifiedType, to: QualifiedType) -> Option<&'static str> {
        match u32::cmp(&sema().layout(&from.id).size, &sema().layout(&to.id).size) {
            Ordering::Less => Some("fpext"),
            Ordering::Greater => Some("fptrunc"),
            Ordering::Equal => None,
        }
    }

    fn i_to_p(&mut self, v: &mut LlvmSymbol, from: QualifiedType) -> Option<&'static str> {
        if let Some(ext) = Self::i_to_i_size(from, ctx().target.pointer.size) {
            *v = self.b.convert(ext, *v, LlvmType::ptr_size());
        }
        Some("inttoptr")
    }

    pub fn fold_raw(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        match node.id.resolve() {
            Expression::Identifier(_) => self.ident(node),
            Expression::StringLiteral(s) => {
                self.globals.get_literal(s.id).copied().invariant("unregistered string literal")
            }
            Expression::Constant(value_node) => Ok(LlvmSymbol::from(value_node.value)),
            Expression::Binary(op, lhs, rhs) => self.binary(op, lhs, rhs),
            Expression::Unary(op, e) => self.unary(op, e),
            Expression::Assign(op, lhs, rhs) => self.assign(op, lhs, rhs),
            Expression::List(lst) => self.list(lst),
            Expression::Ternary(e, lhs, rhs) => self.ternary(e, lhs, rhs),
            Expression::FunctionCall(f, args) => self.call(f, args),
            Expression::ArraySubscripting(array, idx) => self.array_subscript(array, idx),
            Expression::Member(_, _, _) => todo!(),
            Expression::Cast(_, e) => self.explicit_cast(node, e),
            Expression::ConstantExpression(_) | Expression::SizeofExpr(_) | Expression::SizeofType(_) => {
                Err(Diagnosis::Invariant("non folded constant expression"))
            }
        }
    }

    fn list(&mut self, lst: &[ExpressionNode]) -> Result<LlvmSymbol, Diagnosis> {
        let mut last = LlvmSymbol::from(0);
        for e in lst {
            last = self.fold_expression(e)?;
        }
        Ok(last)
    }

    fn ident(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let sym_id = sema().expr_bindings.get(node.id).invariant("unknown identifier")?;

        if let Some(v) = self.globals.get_function(sym_id.resolve().name.id) {
            return Ok(*v);
        }
        self.locals.get(*sym_id).copied().invariant("identifier without storage")
    }

    fn unary(&mut self, op: &UnaryOp, e: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
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

    fn binary(&mut self, op: &BinaryOp, lhs: &ExpressionNode, rhs: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
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

    fn unary_inc_dec(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let loc = self.fold_raw(operand)?;
        let v_before = self.apply_casts(loc, operand)?;
        let lop = LlvmOperator::unary(op, qty)?;
        let v_after = self.b.binop(lop, v_before, LlvmSymbol::from(1));
        self.b.store(v_after, loc);
        match op {
            UnaryOp::PreDec | UnaryOp::PreInc => Ok(v_after),
            UnaryOp::PostDec | UnaryOp::PostInc => Ok(v_before),
            _ => Err(Diagnosis::Invariant("non inc/dec operator in unary_inc_dec")),
        }
    }

    fn unary_minus(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.fold_expression(operand)?;
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let op = LlvmOperator::binary(&BinaryOp::Sub, qty)?;
        Ok(self.b.binop(op, LlvmSymbol::from(0), v))
    }

    fn unary_logic_not(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.fold_expression(operand)?;
        let v = self.neq_zero(v, qty)?;
        let xor = LlvmOperator::unary(op, qty)?;
        let v = self.b.binop(xor, v, LlvmSymbol::from(true));
        Ok(self.zext_to_int(v))
    }

    fn unary_bitnot(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.fold_expression(operand)?;
        let xor = LlvmOperator::unary(op, qty)?;
        Ok(self.b.binop(xor, v, LlvmSymbol::from(-1)))
    }

    fn unary_deref(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.fold_expression(operand)?;
        Ok(LlvmSymbol::ptr(v.name))
    }

    fn binary_arithmetic(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let sema = sema();
        let rel = &sema.expr_types[lhs.id];
        let qty = rel.casted_ty();
        let op = LlvmOperator::binary(op, qty)?;
        let v1 = self.fold_expression(lhs)?;
        let v2 = self.fold_expression(rhs)?;
        Ok(self.b.binop(op, v1, v2))
    }

    fn binary_comparison(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let rel = &sema().expr_types[lhs.id];
        let op = LlvmOperator::binary(op, rel.casted_ty())?;
        let v1 = self.fold_expression(lhs)?;
        let v2 = self.fold_expression(rhs)?;
        let v = self.b.cmp(op, v1, v2);
        Ok(self.zext_to_int(v))
    }

    fn binary_logical(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let initial_block = self.b.current_block;
        let v = self.fold_expression(lhs)?;
        let v = self.neq_zero(v, sema().expr_types[lhs.id].casted_ty())?;
        let i = v.name.ssa_value()?;
        let l1 = LlvmName::label(i, 1);
        let l2 = LlvmName::label(i, 2);
        match op {
            BinaryOp::LogicalAnd => self.b.br(v, l1, Some(l2)),
            BinaryOp::LogicalOr => self.b.br(v, l2, Some(l1)),
            _ => return Err(Diagnosis::Invariant("non logical operator in binary_logical")),
        }

        self.b.named_label(l1);
        let v = self.fold_expression(rhs)?;
        let v = self.neq_zero(v, sema().expr_types[rhs.id].casted_ty())?;
        let rhs_block = self.b.current_block;
        self.b.br(v, l2, None);

        self.b.named_label(l2);
        let v = self.b.phi((*op == BinaryOp::LogicalOr).into(), initial_block, v, rhs_block);
        Ok(self.zext_to_int(v))
    }

    fn assign(
        &mut self,
        op: &Option<BinaryOp>,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let rl = &sema().expr_types[lhs.id];
        let qty = rl.casted_ty();
        let loc = self.fold_raw(lhs)?;
        let mut vr = self.fold_expression(rhs)?;
        if let Some(op) = op {
            let vl = self.apply_casts(loc, lhs)?;
            let op = LlvmOperator::binary(op, qty)?;
            vr = self.b.binop(op, vl, vr);
        }
        self.b.store(vr, loc);
        Ok(vr)
    }

    fn ternary(
        &mut self,
        cond: &ExpressionNode,
        a: &ExpressionNode,
        b: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.fold_expression(cond)?;
        let v = self.neq_zero(v, sema().expr_types[cond.id].casted_ty())?;

        let i = v.name.ssa_value()?;
        let l1 = LlvmName::label(i, 0);
        let l2 = LlvmName::label(i, 1);
        let l3 = LlvmName::label(i, 2);
        self.b.br(v, l1, Some(l2));

        self.b.named_label(l1);
        let va = self.fold_expression(a)?;
        let a_block = self.b.current_block;
        self.b.br(v, l3, None);

        self.b.named_label(l2);
        let vb = self.fold_expression(b)?;
        let b_block = self.b.current_block;
        self.b.br(v, l3, None);

        self.b.named_label(l3);
        Ok(self.b.phi(va, a_block, vb, b_block))
    }

    fn call(&mut self, f: &ExpressionNode, _args: &[ExpressionNode]) -> Result<LlvmSymbol, Diagnosis> {
        let f = self.fold_expression(f)?;
        Ok(self.b.call(f))
    }

    fn array_subscript(&mut self, array: &ExpressionNode, idx: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let arr = self.fold_expression(array)?;
        let idx1 = LlvmSymbol::idx(0);
        let i = sema().expr_consts[idx.id].get_integer_value().invariant("non-integer subscript")?;
        let idx2 = LlvmSymbol::idx(i);

        Ok(self.b.getelementptr(arr, idx1, idx2))
    }

    fn neq_zero(&mut self, v: LlvmSymbol, qty: QualifiedType) -> Result<LlvmSymbol, Diagnosis> {
        let op = LlvmOperator::binary(&BinaryOp::Neq, qty)?;
        Ok(self.b.cmp(op, v, LlvmSymbol::zero(qty)))
    }

    fn zext_to_int(&mut self, v: LlvmSymbol) -> LlvmSymbol {
        self.b.convert("zext", v, LlvmType::int())
    }

    fn explicit_cast(&mut self, node: &ExpressionNode, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.fold_expression(operand)?;
        if sema().expr_types[node.id].ty.is_void(sema()) {
            return Ok(LlvmSymbol::void());
        }
        Ok(v)
    }
}
