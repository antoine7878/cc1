use crate::codegen::{Frozen, LlvmType};
use crate::semantic::{QualifiedType, TagDef, sema};

#[derive(Debug, Clone, Copy)]
pub enum LlvmElement {
    Pad(u32),
    Member { index: usize, ty: QualifiedType },
    Bits { start: u32, bytes: u32, first: usize, last: usize },
}

impl LlvmElement {
    pub fn ty(&self) -> String {
        match self {
            LlvmElement::Pad(n) | LlvmElement::Bits { bytes: n, .. } => Self::bytes_type(*n),
            LlvmElement::Member { ty, .. } => ty.llvm().to_string(),
        }
    }

    pub fn bytes_type(n: u32) -> String {
        match n {
            1 | 2 | 4 | 8 => LlvmType::integer(n).to_string(),
            n => format!("[{n} x {}]", LlvmType::char()),
        }
    }

    pub fn zero(&self) -> String {
        match self {
            LlvmElement::Pad(1 | 2 | 4 | 8) | LlvmElement::Bits { bytes: 1 | 2 | 4 | 8, .. } => {
                format!("{} 0", self.ty())
            }
            _ => format!("{} zeroinitializer", self.ty()),
        }
    }
}

pub fn struct_elements(def: &TagDef, size: u32) -> Vec<LlvmElement> {
    let mut elements = Vec::new();
    let mut cur = 0;
    let pad = |elements: &mut Vec<LlvmElement>, cur: &mut u32, to: u32| {
        if to > *cur {
            elements.push(LlvmElement::Pad(to - *cur));
            *cur = to;
        }
    };
    for (index, member) in def.members.iter().enumerate() {
        match member.width {
            Some(0) => continue,
            None => {
                let Some(id) = member.symbol else { unreachable!() };
                let ty = id.resolve().ty;
                pad(&mut elements, &mut cur, member.offset);
                elements.push(LlvmElement::Member { index, ty });
                cur = member.offset + sema().layout(&ty.id).size;
            }
            Some(width) => {
                let begin = member.offset * 8 + member.bit_offset;
                let end = (begin + width.max(0) as u32).div_ceil(8);
                match elements.last_mut() {
                    Some(LlvmElement::Bits { bytes, last, .. }) if begin / 8 < cur => {
                        *last = index;
                        if end > cur {
                            *bytes += end - cur;
                            cur = end;
                        }
                    }
                    _ => {
                        pad(&mut elements, &mut cur, begin / 8);
                        elements.push(LlvmElement::Bits { start: cur, bytes: end - cur, first: index, last: index });
                        cur = end;
                    }
                }
            }
        }
    }
    pad(&mut elements, &mut cur, size);
    elements
}
