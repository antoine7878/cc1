use crate::codegen::LlvmType;
use crate::semantic::{QualifiedType, TagDef, sema};

#[derive(Debug, Clone, Copy)]
pub enum LlvmElement {
    Pad(u32),
    Member(usize, QualifiedType),
    Bits(u32),
}

impl LlvmElement {
    pub fn ty(&self) -> String {
        match self {
            LlvmElement::Pad(n @ (1 | 2 | 4 | 8)) | LlvmElement::Bits(n @ (1 | 2 | 4 | 8)) => {
                LlvmType::integer(*n).to_string()
            }
            LlvmElement::Pad(n) | LlvmElement::Bits(n) => format!("[{n} x {}]", LlvmType::char()),
            LlvmElement::Member(_, qty) => qty.llvm().to_string(),
        }
    }

    pub fn zero(&self) -> String {
        match self {
            LlvmElement::Pad(1 | 2 | 4 | 8) | LlvmElement::Bits(1 | 2 | 4 | 8) => format!("{} 0", self.ty()),
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
                let Some(id) = member.sym else { unreachable!() };
                let qty = id.resolve().ty;
                pad(&mut elements, &mut cur, member.offset);
                elements.push(LlvmElement::Member(index, qty));
                cur = member.offset + sema().layout(&qty.id).size;
            }
            Some(width) => {
                let end = member.offset + (member.bit_offset + width.max(0) as u32).div_ceil(8);
                if end > cur {
                    pad(&mut elements, &mut cur, member.offset);
                    elements.push(LlvmElement::Bits(end - cur));
                    cur = end;
                }
            }
        }
    }
    pad(&mut elements, &mut cur, size);
    elements
}
