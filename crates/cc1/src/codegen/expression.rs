use std::cmp::Ordering;
use std::io::Write;

use crate::ast::{BinaryOp, ConstValue, Expression, ExpressionNode, UnaryOp};
use crate::codegen::{Generator, Invariant, LlvmOperator, LlvmSymbol, LlvmType};
use crate::context::ctx;
use crate::semantic::{CastKind, Diagnosis, ImplicitCast, QualifiedType, sema};

impl<W: Write> Generator<W> {
    pub fn emit_expression(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let s = match sema().expr_consts.get(node.id) {
            Some(value) => self.constant(sema().expr_types[node.id].ty, *value),
            None => self.fold_raw(node)?,
        };
        self.apply_casts(s, node)
    }

    pub fn constant(&mut self, qty: QualifiedType, value: ConstValue) -> LlvmSymbol {
        if !qty.is_pointer(sema()) {
            return LlvmSymbol::cst(qty.llvm(), value);
        }
        if value.is_zero() {
            return LlvmSymbol::null();
        }
        self.b.convert("inttoptr", LlvmSymbol::cst(LlvmType::ptr_size(), value), qty.llvm())
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
            last = self.emit_expression(e)?;
        }
        Ok(last)
    }

    fn ident(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let &sym_id = sema().expr_bindings.get(node.id).invariant("unknown identifier")?;
        if let Some(v) = self.globals.get_symbol(sym_id) {
            return Ok(*v);
        }
        self.locals.get(sym_id).copied().invariant("unknown identifier")
    }

    fn unary(&mut self, op: &UnaryOp, e: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        match op {
            UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec => self.unary_inc_dec(op, e),
            UnaryOp::Plus => self.emit_expression(e),
            UnaryOp::Minus => self.unary_minus(e),
            UnaryOp::LogicalNot => self.unary_logic_not(e),
            UnaryOp::BitNot => self.unary_bitnot(op, e),
            UnaryOp::Addr => self.emit_expression(e),
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
        let bop = match op {
            UnaryOp::PreInc | UnaryOp::PostInc => BinaryOp::Add,
            UnaryOp::PreDec | UnaryOp::PostDec => BinaryOp::Sub,
            _ => return Err(Diagnosis::Invariant("non inc/dec operator in unary_inc_dec")),
        };
        let one = LlvmSymbol::one(qty);
        let v_after = self.arithmetic(&bop, v_before, qty, one, QualifiedType::plain(sema().builtins.int))?;
        self.b.store(v_after, loc);
        match op {
            UnaryOp::PreDec | UnaryOp::PreInc => Ok(v_after),
            _ => Ok(v_before),
        }
    }

    fn unary_minus(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.emit_expression(operand)?;
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        if qty.is_floating(sema()) {
            return Ok(self.b.unop("fneg", v));
        }
        let op = LlvmOperator::binary(&BinaryOp::Sub, qty)?;
        Ok(self.b.binop(op, LlvmSymbol::cst(v.ty, ConstValue::Int(0)), v))
    }

    fn unary_logic_not(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.logic_not(operand)?;
        Ok(self.zext_to_int(v))
    }

    pub fn logic_not(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.emit_condition(operand)?;
        Ok(self.b.binop("xor", v, LlvmSymbol::from(true)))
    }

    fn unary_bitnot(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let re_operand = &sema().expr_types[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.emit_expression(operand)?;
        let xor = LlvmOperator::unary(op, qty)?;
        Ok(self.b.binop(xor, v, LlvmSymbol::from(-1)))
    }

    fn unary_deref(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.emit_expression(operand)?;
        Ok(LlvmSymbol::ptr(v.name))
    }

    fn binary_arithmetic(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let t1 = sema().expr_types[lhs.id].casted_ty();
        let t2 = sema().expr_types[rhs.id].casted_ty();
        let v1 = self.emit_expression(lhs)?;
        let v2 = self.emit_expression(rhs)?;
        self.arithmetic(op, v1, t1, v2, t2)
    }

    fn arithmetic(
        &mut self,
        op: &BinaryOp,
        v1: LlvmSymbol,
        t1: QualifiedType,
        v2: LlvmSymbol,
        t2: QualifiedType,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let sema = sema();
        match (op, t1.is_pointer(sema), t2.is_pointer(sema)) {
            (BinaryOp::Sub, true, true) => self.pointer_difference(v1, v2, t1),
            (BinaryOp::Add | BinaryOp::Sub, true, false) => self.pointer_offset(v1, t1, v2, *op == BinaryOp::Sub),
            (BinaryOp::Add, false, true) => self.pointer_offset(v2, t2, v1, false),
            _ => Ok(self.b.binop(LlvmOperator::binary(op, t1)?, v1, v2)),
        }
    }

    fn pointer_offset(
        &mut self,
        base: LlvmSymbol,
        ty: QualifiedType,
        mut idx: LlvmSymbol,
        negate: bool,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let elem = ty.id.resolve().pointee().invariant("pointer arithmetic on non-pointer")?;
        if negate {
            idx = self.b.binop("sub", LlvmSymbol::cst(idx.ty, ConstValue::Int(0)), idx);
        }
        Ok(self.b.gep(elem.llvm(), base, idx))
    }

    fn pointer_difference(
        &mut self,
        v1: LlvmSymbol,
        v2: LlvmSymbol,
        ty: QualifiedType,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let elem = ty.id.resolve().pointee().invariant("pointer difference on non-pointer")?;
        let a = self.b.convert("ptrtoint", v1, LlvmType::ptr_size());
        let b = self.b.convert("ptrtoint", v2, LlvmType::ptr_size());
        let d = self.b.binop("sub", a, b);
        let size = sema().layout(&elem.id).size;
        if size == 1 {
            return Ok(d);
        }
        Ok(self.b.binop("sdiv exact", d, LlvmSymbol::idx(u64::from(size))))
    }

    fn binary_comparison(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.comparison(op, lhs, rhs)?;
        Ok(self.zext_to_int(v))
    }

    pub fn comparison(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let rel = &sema().expr_types[lhs.id];
        let op = LlvmOperator::binary(op, rel.casted_ty())?;
        let v1 = self.emit_expression(lhs)?;
        let v2 = self.emit_expression(rhs)?;
        Ok(self.b.cmp(op, v1, v2))
    }

    fn binary_logical(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.logical(op, lhs, rhs)?;
        Ok(self.zext_to_int(v))
    }

    pub fn logical(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnosis> {
        let l1 = self.b.fresh_label();
        let l2 = self.b.fresh_label();
        let initial_block = self.b.current_block;
        let v = self.emit_condition(lhs)?;
        match op {
            BinaryOp::LogicalAnd => self.b.brc(v, l1, l2),
            BinaryOp::LogicalOr => self.b.brc(v, l2, l1),
            _ => return Err(Diagnosis::Invariant("non logical operator in binary_logical")),
        }

        self.b.emit_label(l1);
        let v = self.emit_condition(rhs)?;
        let rhs_block = self.b.current_block;
        self.b.br(l2);

        self.b.emit_label(l2);
        Ok(self.b.phi((*op == BinaryOp::LogicalOr).into(), initial_block, v, rhs_block))
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
        let mut vr = self.emit_expression(rhs)?;
        if let Some(op) = op {
            let vl = self.apply_casts(loc, lhs)?;
            vr = self.arithmetic(op, vl, qty, vr, sema().expr_types[rhs.id].casted_ty())?;
            if let Some(cast) = &rl.result_cast {
                vr = self.convert(vr, qty, cast)?;
            }
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
        let l1 = self.b.fresh_label();
        let l2 = self.b.fresh_label();
        let l3 = self.b.fresh_label();

        let cond = self.emit_condition(cond)?;
        self.b.brc(cond, l1, l2);
        self.b.emit_label(l1);
        let va = self.emit_expression(a)?;
        let a_block = self.b.current_block;
        self.b.br(l3);
        self.b.emit_label(l2);
        let vb = self.emit_expression(b)?;
        let b_block = self.b.current_block;
        self.b.br(l3);
        self.b.emit_label(l3);
        match va.ty.is_void() {
            true => Ok(va),
            false => Ok(self.b.phi(va, a_block, vb, b_block)),
        }
    }

    fn call(&mut self, f: &ExpressionNode, args: &[ExpressionNode]) -> Result<LlvmSymbol, Diagnosis> {
        let f = self.emit_expression(f)?;
        let args = args.iter().map(|e| self.emit_expression(e)).collect::<Result<Vec<_>, Diagnosis>>()?;
        Ok(self.b.call(f, args.as_slice()))
    }

    fn array_subscript(&mut self, array: &ExpressionNode, idx: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let t1 = sema().expr_types[array.id].casted_ty();
        let t2 = sema().expr_types[idx.id].casted_ty();
        let v1 = self.emit_expression(array)?;
        let v2 = self.emit_expression(idx)?;
        self.arithmetic(&BinaryOp::Add, v1, t1, v2, t2)
    }

    pub fn neq_zero(&mut self, v: LlvmSymbol, qty: QualifiedType) -> Result<LlvmSymbol, Diagnosis> {
        let op = LlvmOperator::binary(&BinaryOp::Neq, qty)?;
        Ok(self.b.cmp(op, v, LlvmSymbol::zero(qty)))
    }

    fn zext_to_int(&mut self, v: LlvmSymbol) -> LlvmSymbol {
        self.b.convert("zext", v, LlvmType::int())
    }

    fn explicit_cast(&mut self, node: &ExpressionNode, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let v = self.emit_expression(operand)?;
        if sema().expr_types[node.id].ty.is_void(sema()) {
            return Ok(LlvmSymbol::void());
        }
        Ok(v)
    }
}
