use std::cmp::Ordering;
use std::io::Write;

use crate::ast::{BinaryOp, ConstValue, Expression, ExpressionNode, UnaryOp};
use crate::codegen::{
    BitField, Frozen, Generator, Invariant, LlvmOperator, LlvmParam, LlvmSymbol, LlvmType, ParamAttr, ReturnAttr,
    classify_param,
};
use crate::semantic::{
    CastKind, Diagnostic, ImplicitCast, QualifiedType, ResolvedExpression, ResolvedType, ResolvedTypeId, ValueCategory,
    sema,
};

impl<W: Write> Generator<W> {
    pub fn emit_expression(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let s = match sema().expr_consts.get(node.id) {
            Some(value) => self.emit_constant(sema().expressions[node.id].ty, *value),
            None => self.emit_operation(node)?,
        };
        self.apply_casts(s, node)
    }

    pub fn emit_constant(&mut self, qty: QualifiedType, value: ConstValue) -> LlvmSymbol {
        if !qty.is_pointer(sema()) {
            return LlvmSymbol::cst(qty.llvm(), value);
        }
        if value.is_zero() {
            return LlvmSymbol::null();
        }
        self.builder.convert("inttoptr", LlvmSymbol::cst(LlvmType::int(), value), qty.llvm())
    }

    fn apply_casts(&mut self, mut s: LlvmSymbol, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let re = &sema().expressions[node.id];
        let mut from = re.ty;
        let bf = Self::bitfield_of(node);
        for cast in &re.casts {
            s = self.convert(s, from, cast, bf)?;
            from = cast.to;
        }
        Ok(s)
    }

    fn convert(
        &mut self,
        v: LlvmSymbol,
        from: QualifiedType,
        cast: &ImplicitCast,
        bf: Option<BitField>,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let mut v = v;
        let to = cast.to;
        let conv = match cast.kind {
            CastKind::ArrayToPointer | CastKind::FunctionToPointer | CastKind::PointerConversion => return Ok(v),
            CastKind::LValueToRValue if to.is_record(sema()) => return Ok(v),
            CastKind::LValueToRValue => return Ok(self.emit_load(v, to, bf.as_ref())),
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
            Some(conv) => self.builder.convert(conv, v, to.llvm()),
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
        if let Some(ext) = Self::i_to_i_size(from, LlvmType::int().size()) {
            *v = self.builder.convert(ext, *v, LlvmType::int());
        }
        Some("inttoptr")
    }

    pub fn emit_operation(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        match node.id.resolve() {
            Expression::Identifier(_) => self.identifier(node),
            Expression::StringLiteral(s) => {
                self.globals.literal(s.id).copied().invariant("unregistered string literal")
            }
            Expression::Constant(value_node) => Ok(LlvmSymbol::from(value_node.value)),
            Expression::Binary(op, lhs, rhs) => self.binary(op, lhs, rhs),
            Expression::Unary(op, e) => self.unary(op, e),
            Expression::Assign(op, lhs, rhs) => self.assign(op, lhs, rhs),
            Expression::List(items) => self.list(items),
            Expression::Ternary(e, lhs, rhs) => self.ternary(e, lhs, rhs),
            Expression::FunctionCall(f, args) => self.call(node, f, args),
            Expression::ArraySubscripting(array, idx) => self.array_subscript(array, idx),
            Expression::Member(_, object, _) => self.member(node, object),
            Expression::Cast(_, e) => self.explicit_cast(node, e),
            Expression::ConstantExpression(_) | Expression::SizeofExpr(_) | Expression::SizeofType(_) => {
                Err(Diagnostic::Invariant("non folded constant expression"))
            }
        }
    }

    fn list(&mut self, items: &[ExpressionNode]) -> Result<LlvmSymbol, Diagnostic> {
        let mut last = LlvmSymbol::from(0);
        for e in items {
            last = self.emit_expression(e)?;
        }
        Ok(last)
    }

    fn identifier(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let &sym_id = sema().expr_bindings.get(node.id).invariant("unknown identifier")?;
        if let Some(v) = self.globals.symbol(sym_id) {
            return Ok(*v);
        }
        self.locals.get(sym_id).copied().invariant("unknown identifier")
    }

    fn unary(&mut self, op: &UnaryOp, e: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        match op {
            UnaryOp::PostInc | UnaryOp::PostDec | UnaryOp::PreInc | UnaryOp::PreDec => self.unary_inc_dec(op, e),
            UnaryOp::Plus => self.emit_expression(e),
            UnaryOp::Minus => self.unary_minus(e),
            UnaryOp::LogicalNot => self.unary_logical_not(e),
            UnaryOp::BitNot => self.unary_bit_not(op, e),
            UnaryOp::Addr => self.emit_expression(e),
            UnaryOp::Deref => self.unary_deref(e),
        }
    }

    fn binary(&mut self, op: &BinaryOp, lhs: &ExpressionNode, rhs: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
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

    fn unary_inc_dec(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let re_operand = &sema().expressions[operand.id];
        let qty = re_operand.casted_ty();
        let loc = self.emit_operation(operand)?;
        let v_before = self.apply_casts(loc, operand)?;
        let bop = match op {
            UnaryOp::PreInc | UnaryOp::PostInc => BinaryOp::Add,
            _ => BinaryOp::Sub,
        };
        let one = LlvmSymbol::one(qty);
        let v_after = self.arithmetic(&bop, v_before, qty, one, QualifiedType::plain(sema().builtins.int))?;
        let bf = Self::bitfield_of(operand);
        let v_after = self.emit_store(v_after, loc, bf.as_ref());
        match op {
            UnaryOp::PreDec | UnaryOp::PreInc => Ok(v_after),
            _ => Ok(v_before),
        }
    }

    fn unary_minus(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_expression(operand)?;
        let re_operand = &sema().expressions[operand.id];
        let qty = re_operand.casted_ty();
        if qty.is_floating(sema()) {
            return Ok(self.builder.unop("fneg", v));
        }
        let op = LlvmOperator::binary(&BinaryOp::Sub, qty)?;
        Ok(self.builder.binop(op, LlvmSymbol::cst(v.ty, ConstValue::Int(0)), v))
    }

    fn unary_logical_not(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_logical_not(operand)?;
        Ok(self.zext_to_int(v))
    }

    pub fn emit_logical_not(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_condition(operand)?;
        Ok(self.builder.binop("xor", v, LlvmSymbol::from(true)))
    }

    fn unary_bit_not(&mut self, op: &UnaryOp, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let re_operand = &sema().expressions[operand.id];
        let qty = re_operand.casted_ty();
        let v = self.emit_expression(operand)?;
        let xor = LlvmOperator::unary(op, qty)?;
        Ok(self.builder.binop(xor, v, LlvmSymbol::from(-1)))
    }

    fn unary_deref(&mut self, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_expression(operand)?;
        Ok(LlvmSymbol::ptr(v.name))
    }

    fn binary_arithmetic(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let t1 = sema().expressions[lhs.id].casted_ty();
        let t2 = sema().expressions[rhs.id].casted_ty();
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
    ) -> Result<LlvmSymbol, Diagnostic> {
        let sema = sema();
        match (op, t1.is_pointer(sema), t2.is_pointer(sema)) {
            (BinaryOp::Sub, true, true) => self.pointer_difference(v1, v2, t1),
            (BinaryOp::Add | BinaryOp::Sub, true, false) => self.pointer_offset(v1, t1, v2, *op == BinaryOp::Sub),
            (BinaryOp::Add, false, true) => self.pointer_offset(v2, t2, v1, false),
            _ => Ok(self.builder.binop(LlvmOperator::binary(op, t1)?, v1, v2)),
        }
    }

    fn pointer_offset(
        &mut self,
        base: LlvmSymbol,
        ty: QualifiedType,
        mut idx: LlvmSymbol,
        negate: bool,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let elem = ty.id.resolve().pointee().invariant("pointer arithmetic on non-pointer")?;
        if negate {
            idx = self.builder.binop("sub", LlvmSymbol::cst(idx.ty, ConstValue::Int(0)), idx);
        }
        Ok(self.builder.gep(elem.llvm(), base, idx))
    }

    fn pointer_difference(
        &mut self,
        v1: LlvmSymbol,
        v2: LlvmSymbol,
        ty: QualifiedType,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let elem = ty.id.resolve().pointee().invariant("pointer difference on non-pointer")?;
        let a = self.builder.convert("ptrtoint", v1, LlvmType::int());
        let b = self.builder.convert("ptrtoint", v2, LlvmType::int());
        let d = self.builder.binop("sub", a, b);
        let size = sema().layout(&elem.id).size;
        if size == 1 {
            return Ok(d);
        }
        Ok(self.builder.binop("sdiv exact", d, LlvmSymbol::idx(u64::from(size))))
    }

    fn binary_comparison(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_comparison(op, lhs, rhs)?;
        Ok(self.zext_to_int(v))
    }

    pub fn emit_comparison(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let rel = &sema().expressions[lhs.id];
        let op = LlvmOperator::binary(op, rel.casted_ty())?;
        let v1 = self.emit_expression(lhs)?;
        let v2 = self.emit_expression(rhs)?;
        Ok(self.builder.cmp(op, v1, v2))
    }

    fn binary_logical(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_logical(op, lhs, rhs)?;
        Ok(self.zext_to_int(v))
    }

    pub fn emit_logical(
        &mut self,
        op: &BinaryOp,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let l1 = self.builder.fresh_label();
        let l2 = self.builder.fresh_label();
        let v = self.emit_condition(lhs)?;
        let lhs_block = self.builder.current_block;
        match op {
            BinaryOp::LogicalAnd => self.builder.br_cond(v, l1, l2),
            BinaryOp::LogicalOr => self.builder.br_cond(v, l2, l1),
            _ => return Err(Diagnostic::Invariant("non logical operator in binary_logical")),
        }

        self.builder.label(l1);
        let v = self.emit_condition(rhs)?;
        let rhs_block = self.builder.current_block;
        self.builder.br(l2);

        self.builder.label(l2);
        Ok(self.builder.phi((*op == BinaryOp::LogicalOr).into(), lhs_block, v, rhs_block))
    }

    fn assign(
        &mut self,
        op: &Option<BinaryOp>,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let rl = &sema().expressions[lhs.id];
        let qty = rl.casted_ty();

        let loc = self.emit_operation(lhs)?;
        match qty.is_record(sema()) {
            true => self.emit_copy_aggregate(loc, rhs, qty),
            false => self.copy_scalar(op, loc, lhs, rhs, rl),
        }
    }

    fn copy_scalar(
        &mut self,
        op: &Option<BinaryOp>,
        loc: LlvmSymbol,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
        rl: &ResolvedExpression,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let mut vr = self.emit_expression(rhs)?;

        if let Some(op) = op {
            let vl = self.apply_casts(loc, lhs)?;
            vr = self.arithmetic(op, vl, rl.casted_ty(), vr, sema().expressions[rhs.id].casted_ty())?;
            if let Some(cast) = &rl.result_cast {
                vr = self.convert(vr, rl.casted_ty(), cast, None)?;
            }
        }
        let bf = Self::bitfield_of(lhs);
        Ok(self.emit_store(vr, loc, bf.as_ref()))
    }

    pub fn emit_copy_aggregate(
        &mut self,
        loc: LlvmSymbol,
        rhs: &ExpressionNode,
        qty: QualifiedType,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let src = self.emit_expression(rhs)?;
        self.builder.memcpy(loc.name, src.name, sema().layout(&qty.id));
        Ok(loc)
    }

    fn ternary(
        &mut self,
        cond: &ExpressionNode,
        a: &ExpressionNode,
        b: &ExpressionNode,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let l1 = self.builder.fresh_label();
        let l2 = self.builder.fresh_label();
        let l3 = self.builder.fresh_label();

        let cond = self.emit_condition(cond)?;
        self.builder.br_cond(cond, l1, l2);
        self.builder.label(l1);
        let va = self.emit_expression(a)?;
        let a_block = self.builder.current_block;
        self.builder.br(l3);
        self.builder.label(l2);
        let vb = self.emit_expression(b)?;
        let b_block = self.builder.current_block;
        self.builder.br(l3);
        self.builder.label(l3);
        match va.ty.is_void() {
            true => Ok(va),
            false => Ok(self.builder.phi(va, a_block, vb, b_block)),
        }
    }

    fn call(
        &mut self,
        node: &ExpressionNode,
        f: &ExpressionNode,
        args: &[ExpressionNode],
    ) -> Result<LlvmSymbol, Diagnostic> {
        let qty =
            sema().expressions[f.id].casted_ty().id.resolve().pointee().invariant("call of non-pointer function")?;
        let fty = LlvmType::Function(qty.id);
        let ResolvedType::Function { ret, .. } = qty.id.resolve() else {
            return Err(Diagnostic::Invariant("call of non-function"));
        };
        let ret_attr = ReturnAttr::classify_return(*ret);
        let f = self.emit_expression(f)?;
        let mut params: Vec<LlvmParam> = vec![];

        if let ReturnAttr::Sret { ty, align } = ret_attr {
            let slot = self.locals.spill(node.id);
            params.push(LlvmParam { sym: slot, attr: ParamAttr::SRet { ty, align } })
        }
        for e in args {
            params.push(self.emit_argument(e)?);
        }
        let v = self.builder.call(fty, ret_attr, f, params.as_slice());
        match ret_attr {
            ReturnAttr::Void | ReturnAttr::Direct(_) => Ok(v),
            ReturnAttr::Sret { .. } => Ok(self.locals.spill(node.id)),
        }
    }

    fn emit_argument(&mut self, e: &ExpressionNode) -> Result<LlvmParam, Diagnostic> {
        let qty = sema().expressions[e.id].casted_ty();
        let v = self.emit_expression(e)?;
        let (_, attr) = classify_param(qty);
        Ok(LlvmParam::new(v, attr))
    }

    fn array_subscript(&mut self, array: &ExpressionNode, idx: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let t1 = sema().expressions[array.id].casted_ty();
        let t2 = sema().expressions[idx.id].casted_ty();
        let v1 = self.emit_expression(array)?;
        let v2 = self.emit_expression(idx)?;
        self.arithmetic(&BinaryOp::Add, v1, t1, v2, t2)
    }

    pub fn emit_nonzero(&mut self, v: LlvmSymbol, qty: QualifiedType) -> Result<LlvmSymbol, Diagnostic> {
        let op = LlvmOperator::binary(&BinaryOp::Neq, qty)?;
        Ok(self.builder.cmp(op, v, LlvmSymbol::zero(qty)))
    }

    fn zext_to_int(&mut self, v: LlvmSymbol) -> LlvmSymbol {
        self.builder.convert("zext", v, LlvmType::int())
    }

    fn member(&mut self, node: &ExpressionNode, object_node: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let re = &sema().expressions[object_node.id];
        let object = self.emit_expression(object_node)?;
        let ty = re.ty.id;
        let place = match ty.resolve() {
            ResolvedType::Pointer(qty) => self.member_place(node, object, qty.id)?,
            ResolvedType::Tag(_) => self.member_place(node, object, ty)?,
            _ => unreachable!(),
        };
        let &ResolvedExpression { ty: qty, kind, .. } = &sema().expressions[node.id];
        match kind == ValueCategory::RValue && qty.is_scalar(sema()) {
            true => Ok(self.emit_load(place, qty, Self::bitfield_of(node).as_ref())),
            false => Ok(place),
        }
    }

    fn member_place(
        &mut self,
        node: &ExpressionNode,
        base: LlvmSymbol,
        ty: ResolvedTypeId,
    ) -> Result<LlvmSymbol, Diagnostic> {
        let ResolvedType::Tag(tag_id) = ty.resolve() else { unreachable!() };
        let tagdef = tag_id.resolve();
        let member_idx = sema().member_refs[node.id].index;
        let idx = LlvmSymbol::cst(LlvmType::int(), ConstValue::Long(tagdef.members[member_idx].offset as i64));
        Ok(self.builder.gep(LlvmType::I8, base, idx))
    }

    fn explicit_cast(&mut self, node: &ExpressionNode, operand: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let v = self.emit_expression(operand)?;
        if sema().expressions[node.id].ty.is_void(sema()) {
            return Ok(LlvmSymbol::void());
        }
        Ok(v)
    }
}
