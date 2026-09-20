use std::fmt::{self, Display, Formatter};
use std::ops::RangeInclusive;

use crate::ast::{ConstFolder, ConstValue, StringConstId, Tag};
use crate::codegen::{Frozen, LlvmElement, LlvmName, LlvmSymbol, LlvmType, struct_elements};
use crate::semantic::{AddressBase, AddressOffset, Initializer, QualifiedType, ResolvedType, TagDef, sema};

pub struct LlvmInit<'a> {
    pub ty: QualifiedType,
    pub init: Option<&'a Initializer>,
}

impl<'a> LlvmInit<'a> {
    pub fn new(ty: QualifiedType, init: Option<&'a Initializer>) -> Self {
        Self { ty, init }
    }

    fn zero(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} zeroinitializer", self.ty.llvm())
    }

    fn value(&self, f: &mut Formatter<'_>, value: ConstValue) -> fmt::Result {
        let rty = self.ty.id.resolve();
        if rty.is_pointer() {
            return match value.is_zero() {
                true => write!(f, "{}", LlvmSymbol::null()),
                false => write!(f, "ptr inttoptr ({} to ptr)", LlvmSymbol::cst(LlvmType::int(), value)),
            };
        }
        let rty = match rty {
            ResolvedType::Tag(id) if id.resolve().is_enum() => &ResolvedType::Int,
            rty => rty,
        };
        let value = ConstFolder.convert(rty, value).unwrap_or(value);
        write!(f, "{}", LlvmSymbol::cst(self.ty.llvm(), value))
    }

    fn address(&self, f: &mut Formatter<'_>, place: AddressOffset) -> fmt::Result {
        let rty = self.ty.id.resolve();
        if rty.is_pointer() {
            return write!(f, "ptr {}", LlvmAddress(place));
        }
        let ty = self.ty.llvm();
        write!(f, "{ty} ptrtoint (ptr {} to {ty})", LlvmAddress(place))
    }

    fn string(&self, f: &mut Formatter<'_>, id: StringConstId) -> fmt::Result {
        let units = &id.resolve().units;
        let len = match self.ty.id.resolve() {
            ResolvedType::Array { len, .. } => len.unwrap_or(units.len() + 1),
            _ => unreachable!("string initializer on a non-array"),
        };
        let ty = if id.resolve().is_wide { LlvmType::int() } else { LlvmType::char() };
        write!(f, "[{len} x {ty}] [")?;
        let cells = units.iter().copied().chain(std::iter::repeat(0)).take(len);
        for (i, c) in cells.enumerate() {
            let sep = if i == 0 { "" } else { ", " };
            write!(f, "{sep}{ty} {c}")?;
        }
        write!(f, "]")
    }

    fn list(&self, f: &mut Formatter<'_>, items: &[Initializer]) -> fmt::Result {
        match self.ty.id.resolve() {
            ResolvedType::Array { elem, len } => self.array(f, *elem, len.unwrap_or(items.len()), items),
            ResolvedType::Tag(id) => match id.resolve().kind {
                Tag::Struct => self.structure(f, id.resolve(), items),
                Tag::Union => self.union(f, id.resolve(), items),
                Tag::Enum => unreachable!("list initializer on an enum"),
            },
            _ => unreachable!("list initializer on a scalar"),
        }
    }

    fn array(&self, f: &mut Formatter<'_>, elem: QualifiedType, len: usize, items: &[Initializer]) -> fmt::Result {
        write!(f, "{} [", self.ty.llvm())?;
        for i in 0..len {
            let sep = if i == 0 { "" } else { ", " };
            write!(f, "{sep}{}", LlvmInit::new(elem, items.get(i)))?;
        }
        write!(f, "]")
    }

    fn structure(&self, f: &mut Formatter<'_>, def: &TagDef, items: &[Initializer]) -> fmt::Result {
        let size = sema().layout(&self.ty.id).size;
        write!(f, "{} <{{ ", self.ty.llvm())?;
        for (i, element) in struct_elements(def, size).iter().enumerate() {
            let sep = if i == 0 { "" } else { ", " };
            write!(f, "{sep}")?;
            match *element {
                LlvmElement::Member { index, ty } => {
                    write!(f, "{}", LlvmInit::new(ty, items.get(item_index(def, index))))?
                }
                LlvmElement::Bits { start, bytes, first, last } => {
                    self.bits(f, def, items, start, bytes, first..=last)?
                }
                LlvmElement::Pad(_) => write!(f, "{}", element.zero())?,
            }
        }
        write!(f, " }}>")
    }

    fn bits(
        &self,
        f: &mut Formatter<'_>,
        def: &TagDef,
        items: &[Initializer],
        start: u32,
        bytes: u32,
        members: RangeInclusive<usize>,
    ) -> fmt::Result {
        let mut value: u64 = 0;
        for index in members {
            let member = def.members[index];
            let (Some(sym), Some(width)) = (member.symbol, member.width) else { continue };
            let field = match items.get(item_index(def, index)) {
                None | Some(Initializer::Zero) => 0,
                Some(Initializer::Value(v)) => {
                    let rty = sym.resolve().ty.id.resolve();
                    ConstFolder.convert(rty, *v).unwrap_or(*v).to_u64()
                }
                _ => unreachable!("non-constant bit-field initializer"),
            };
            let shift = member.offset * 8 + member.bit_offset - start * 8;
            let mask = (1u64 << width) - 1;
            value |= (field & mask) << shift;
        }
        match bytes {
            1 | 2 | 4 | 8 => write!(f, "{} {value}", LlvmElement::bytes_type(bytes)),
            n => {
                write!(f, "{} [", LlvmElement::bytes_type(n))?;
                for i in 0..n {
                    let sep = if i == 0 { "" } else { ", " };
                    write!(f, "{sep}{} {}", LlvmType::char(), (value >> (8 * i)) & 0xff)?;
                }
                write!(f, "]")
            }
        }
    }

    fn union(&self, f: &mut Formatter<'_>, def: &TagDef, items: &[Initializer]) -> fmt::Result {
        let Some(first) = def.members.iter().find_map(|member| member.symbol) else {
            return self.zero(f);
        };
        let first = first.resolve().ty;
        let pad = sema().layout(&self.ty.id).size - sema().layout(&first.id).size;
        let init = LlvmInit::new(first, items.first());
        match pad {
            0 => write!(f, "{{ {} }} {{ {init} }}", first.llvm()),
            pad => {
                let padding = format!("[{pad} x {}]", LlvmType::char());
                write!(f, "{{ {}, {padding} }} {{ {init}, {padding} zeroinitializer }}", first.llvm())
            }
        }
    }
}

impl Display for LlvmInit<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.init {
            None | Some(Initializer::Zero) => self.zero(f),
            Some(Initializer::Value(value)) => self.value(f, *value),
            Some(Initializer::Address(place)) => self.address(f, *place),
            Some(Initializer::String(id)) => self.string(f, *id),
            Some(Initializer::List(items)) => self.list(f, items),
            Some(Initializer::Expr(_)) => unreachable!("expression initializer on a static object"),
        }
    }
}

struct LlvmAddress(AddressOffset);

impl Display for LlvmAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let offset = LlvmSymbol::cst(LlvmType::int(), ConstValue::Long(self.0.offset));
        let base = match self.0.base {
            AddressBase::Symbol(sym) => LlvmName::Global(sym),
            AddressBase::String(id) => LlvmName::StringLiteral(id),
            AddressBase::Absolute => return write!(f, "inttoptr ({offset} to ptr)"),
        };
        match self.0.offset {
            0 => write!(f, "{base}"),
            _ => write!(f, "getelementptr ({}, ptr {base}, {offset})", LlvmType::char()),
        }
    }
}

fn item_index(def: &TagDef, index: usize) -> usize {
    def.members[..index].iter().filter(|member| member.symbol.is_some()).count()
}
