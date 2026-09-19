use std::io::Write;

use crate::{
    ast::{ConstValue, Expression, ExpressionNode},
    codegen::{Generator, LlvmSymbol, LlvmType},
    semantic::{QualifiedType, sema},
};

#[derive(Clone, Copy)]
pub struct BitField {
    unit: LlvmType,
    bit_offset: i32,
    width: i32,
    signed: bool,
}

impl<W: Write> Generator<W> {
    pub fn bitfield_of(node: &ExpressionNode) -> Option<BitField> {
        let Expression::Member(_, _, _) = node.id.resolve() else { return None };
        let member = sema().member_refs[node.id].member(sema());
        let width = member.width?;
        let qty = sema().expressions[node.id].ty;
        Some(BitField {
            unit: qty.llvm(),
            bit_offset: member.bit_offset as i32,
            width,
            signed: !qty.is_unsigned(sema()),
        })
    }

    pub fn emit_load(&mut self, loc: LlvmSymbol, qty: QualifiedType, bf: Option<&BitField>) -> LlvmSymbol {
        match bf {
            None => self.builder.load(qty.llvm(), loc),
            Some(bf) => {
                let v = self.builder.load(bf.unit, loc);
                self.extract(v, bf)
            }
        }
    }

    pub fn emit_store(&mut self, src: LlvmSymbol, loc: LlvmSymbol, bf: Option<&BitField>) -> LlvmSymbol {
        match bf {
            None => self.builder.store(src, loc),
            Some(bf) => {
                let old = self.builder.load(bf.unit, loc);
                let mask = ((1u64 << bf.width) - 1) << bf.bit_offset;

                let cst = LlvmSymbol::cst(bf.unit, ConstValue::Int(!mask as u32 as i32));
                let cleared = self.builder.binop("and", old, cst);

                let cst = LlvmSymbol::cst(bf.unit, ConstValue::Int(bf.bit_offset));
                let v = self.builder.binop("shl", src, cst);
                let cst = LlvmSymbol::cst(bf.unit, ConstValue::Int(mask as u32 as i32));
                let field = self.builder.binop("and", v, cst);

                let v = self.builder.binop("or", cleared, field);
                self.builder.store(v, loc);
                self.extract(v, bf)
            }
        }
    }

    fn extract(&mut self, v: LlvmSymbol, bf: &BitField) -> LlvmSymbol {
        let bits = (bf.unit.size() * 8) as i32;
        let cst = LlvmSymbol::cst(bf.unit, ConstValue::Int(bits - bf.width - bf.bit_offset));
        let sh = self.builder.binop("shl", v, cst);
        let op = if bf.signed { "ashr" } else { "lshr" };
        let cst = LlvmSymbol::cst(bf.unit, ConstValue::Int(bits - bf.width));
        self.builder.binop(op, sh, cst)
    }
}
