use std::fmt::{self, Display, Formatter};
use std::ops::RangeInclusive;

use crate::ast::{ConstFolder, ConstValue, StringConstId, Tag};
use crate::codegen::{Frozen, LlvmElement, LlvmName, LlvmSymbol, LlvmType, struct_elements};
use crate::semantic::{AddressBase, AddressOffset, Initializer, QualifiedType, ResolvedType, TagDef, sema};

const COMPACT_TAIL: usize = 64;

pub struct LlvmInit<'a> {
    pub ty: QualifiedType,
    pub init: Option<&'a Initializer>,
    top: bool,
}

impl<'a> LlvmInit<'a> {
    pub fn new(ty: QualifiedType, init: Option<&'a Initializer>) -> Self {
        Self { ty, init, top: false }
    }

    pub fn top(ty: QualifiedType, init: Option<&'a Initializer>) -> Self {
        Self { ty, init, top: true }
    }

    fn compact_tail(&self, used: usize, len: usize) -> Option<usize> {
        (self.top && used > 0 && len - used >= COMPACT_TAIL).then_some(len - used)
    }

    fn zero(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} zeroinitializer", repr(self.ty))
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
            rty => rty.underlying(sema()),
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
        let used = units.len().min(len);
        if used == 0 && len >= COMPACT_TAIL {
            return self.zero(f);
        }
        if let Some(tail) = self.compact_tail(used, len) {
            write!(f, "<{{ [{used} x {ty}], [{tail} x {ty}] }}> <{{ [{used} x {ty}] [")?;
            for (i, c) in units.iter().enumerate() {
                let sep = if i == 0 { "" } else { ", " };
                write!(f, "{sep}{ty} {c}")?;
            }
            return write!(f, "], [{tail} x {ty}] zeroinitializer }}>");
        }
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
        let used = items.iter().rposition(|item| !is_zero_item(item)).map_or(0, |last| last + 1).min(len);
        if used == 0 && len >= COMPACT_TAIL {
            return self.zero(f);
        }
        let ty = repr(elem);
        let tail = self.compact_tail(used, len);
        let shown = if tail.is_some() { used } else { len };
        if let Some(tail) = tail {
            write!(f, "<{{ [{used} x {ty}], [{tail} x {ty}] }}> <{{ ")?;
        }
        write!(f, "[{shown} x {ty}] [")?;
        for i in 0..shown {
            let sep = if i == 0 { "" } else { ", " };
            write!(f, "{sep}{}", LlvmInit::new(elem, items.get(i)))?;
        }
        write!(f, "]")?;
        match tail {
            Some(tail) => write!(f, ", [{tail} x {ty}] zeroinitializer }}>"),
            None => Ok(()),
        }
    }

    fn structure(&self, f: &mut Formatter<'_>, def: &TagDef, items: &[Initializer]) -> fmt::Result {
        let size = sema().layout(&self.ty.id).unwrap().size;
        write!(f, "{} <{{ ", repr(self.ty))?;
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
        let Some(first) = union_first(def) else {
            return self.zero(f);
        };
        let init = LlvmInit::new(first, items.first());
        match union_pad(self.ty, first) {
            0 => write!(f, "{} {{ {init} }}", repr(self.ty)),
            pad => write!(f, "{} {{ {init}, [{pad} x {}] zeroinitializer }}", repr(self.ty), LlvmType::char()),
        }
    }
}

fn is_zero_item(item: &Initializer) -> bool {
    match item {
        Initializer::Zero => true,
        Initializer::Value(value) => !value.is_floating() && value.is_zero(),
        _ => false,
    }
}

pub fn union_first(def: &TagDef) -> Option<QualifiedType> {
    def.members.iter().find_map(|member| member.symbol).map(|sym| sym.resolve().ty)
}

pub fn union_widest(def: &TagDef) -> Option<QualifiedType> {
    let members = def.members.iter().filter_map(|member| member.symbol).map(|sym| sym.resolve().ty);
    members.max_by_key(|qty| sema().layout(&qty.id).unwrap().align)
}

fn union_pad(union: QualifiedType, member: QualifiedType) -> u32 {
    sema().layout(&union.id).unwrap().size - sema().layout(&member.id).unwrap().size
}

fn needs_literal(qty: QualifiedType) -> bool {
    match qty.id.resolve() {
        ResolvedType::Array { elem, .. } => needs_literal(*elem),
        ResolvedType::Tag(id) => {
            let def = id.resolve();
            match def.kind {
                Tag::Struct => def.members.iter().filter_map(|m| m.symbol).any(|sym| needs_literal(sym.resolve().ty)),
                Tag::Union => match (union_first(def), union_widest(def)) {
                    (Some(first), Some(widest)) => first.llvm() != widest.llvm() || needs_literal(first),
                    _ => false,
                },
                Tag::Enum => false,
            }
        }
        _ => false,
    }
}

fn repr(qty: QualifiedType) -> String {
    if !needs_literal(qty) {
        return qty.llvm().to_string();
    }
    match qty.id.resolve() {
        ResolvedType::Array { elem, len } => format!("[{} x {}]", len.unwrap_or(0), repr(*elem)),
        ResolvedType::Tag(id) => {
            let def = id.resolve();
            match def.kind {
                Tag::Struct => {
                    let size = sema().layout(&qty.id).unwrap().size;
                    let elements = struct_elements(def, size)
                        .iter()
                        .map(|element| match *element {
                            LlvmElement::Member { ty, .. } => repr(ty),
                            element => element.ty().to_string(),
                        })
                        .collect::<Vec<_>>();
                    format!("<{{ {} }}>", elements.join(", "))
                }
                Tag::Union => {
                    let first = union_first(def).unwrap();
                    match union_pad(qty, first) {
                        0 => format!("{{ {} }}", repr(first)),
                        pad => format!("{{ {}, [{pad} x {}] }}", repr(first), LlvmType::char()),
                    }
                }
                Tag::Enum => unreachable!("literal type of an enum"),
            }
        }
        _ => unreachable!("literal type of a scalar"),
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
