use crate::arena::ResolveWith;
use crate::ast::Tag;
use crate::semantic::{Member, ResolvedType, ResolvedTypeId, Sema, TagDefId};
use crate::target::Layout;

fn round_up(value: u64, multiple: u64) -> u64 {
    match value % multiple {
        0 => value,
        rest => value + multiple - rest,
    }
}

pub fn of(sema: &mut Sema, qualified_type: ResolvedTypeId) -> Option<Layout> {
    if let Some(&layout) = sema.layouts.get(&qualified_type) {
        return Some(layout);
    }
    let layout = match qualified_type.resolve_in(sema) {
        ResolvedType::Tag(id) => of_tag(sema, *id)?,
        ResolvedType::Array { elem, len } => {
            let len = *len;
            let elem = of(sema, elem.id)?;
            Layout::new(elem.size * len.unwrap_or(0) as u32, elem.align)
        }
        ty => sema.target.layout(ty)?,
    };
    sema.layouts.insert(qualified_type, layout);
    Some(layout)
}

pub fn finalize(sema: &mut Sema) {
    for index in 0..sema.tags.len() {
        let _ = of_tag(sema, TagDefId::from(index));
    }
}

pub fn of_tag(sema: &mut Sema, id: TagDefId) -> Option<Layout> {
    let mut tag = id.resolve_in(sema).clone();
    if !tag.is_complete {
        return None;
    }
    let layout = match tag.kind {
        Tag::Struct => struct_layout(sema, &mut tag.members)?,
        Tag::Union => union_layout(sema, &mut tag.members)?,
        Tag::Enum => return Some(sema.target.int),
    };
    sema.tags.get_mut(id).members = tag.members;
    Some(layout)
}

fn member(sema: &mut Sema, mem: Member) -> Option<(Layout, Option<u64>)> {
    let width = mem.width.map(|width| width.max(0) as u64);
    match mem.sym {
        Some(id) => {
            let ty = id.resolve_in(sema).ty?;
            Some((of(sema, ty.id)?, width))
        }
        None => Some((of(sema, sema.builtins.int)?, width)),
    }
}

fn place_plain(m: &mut Member, bits: u64) {
    m.offset = (bits / 8) as u32;
    m.bit_offset = 0;
}

fn place_bitfield(m: &mut Member, bits: u64, storage: u64) {
    let unit = bits / storage * storage;
    m.offset = (unit / 8) as u32;
    m.bit_offset = (bits - unit) as u32;
}

fn struct_layout(sema: &mut Sema, members: &mut [Member]) -> Option<Layout> {
    let mut bits: u64 = 0;
    let mut align: u32 = 1;

    for m in members.iter_mut() {
        let (layout, width) = member(sema, *m)?;
        let unit = u64::from(layout.align) * 8;
        if m.sym.is_some() {
            align = align.max(layout.align);
        }
        match width {
            Some(0) => {
                bits = round_up(bits, unit);
                place_plain(m, bits);
            }
            Some(width) => {
                let storage = u64::from(layout.size) * 8;
                if bits % storage + width > storage {
                    bits = round_up(bits, unit);
                }
                place_bitfield(m, bits, storage);
                bits += width;
            }
            None => {
                bits = round_up(bits, unit);
                place_plain(m, bits);
                bits += u64::from(layout.size) * 8;
            }
        }
    }

    let size = round_up(bits, u64::from(align) * 8) / 8;
    Some(Layout::new(size as u32, align))
}

fn union_layout(sema: &mut Sema, members: &mut [Member]) -> Option<Layout> {
    let mut size: u64 = 0;
    let mut align: u32 = 1;

    for m in members.iter_mut() {
        let (layout, width) = member(sema, *m)?;
        if m.sym.is_some() {
            align = align.max(layout.align);
        }
        size = size.max(width.map_or(u64::from(layout.size), |width| width.div_ceil(8)));
        m.offset = 0;
        m.bit_offset = 0;
    }

    Some(Layout::new(round_up(size, u64::from(align)) as u32, align))
}
