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
    let layout = match qualified_type.resolve(sema) {
        ResolvedType::Tag(id) => of_tag(sema, *id)?,
        ResolvedType::Array { elem, len } => {
            let len = *len;
            let elem = of(sema, elem.id)?;
            Layout::new(elem.size * len.unwrap_or(0) as u32, elem.align)
        }
        ty => sema.target.scalar(ty)?,
    };
    sema.layouts.insert(qualified_type, layout);
    Some(layout)
}

pub fn of_tag(sema: &mut Sema, id: TagDefId) -> Option<Layout> {
    let tag = id.resolve(sema).clone();
    if !tag.is_complete {
        return None;
    }
    match tag.kind {
        Tag::Struct => struct_layout(sema, &tag.members),
        Tag::Union => union_layout(sema, &tag.members),
        Tag::Enum => Some(sema.target.int),
    }
}

fn member(sema: &mut Sema, mem: Member) -> Option<(Layout, Option<u64>)> {
    match mem {
        Member::Symbol(id) => {
            let symbol = id.resolve(sema);
            let width = symbol.value.map(|width| width.max(0) as u64);
            let ty = symbol.ty?;
            Some((of(sema, ty.id)?, width))
        }
        Member::Bitfield(i) => {
            let ty = sema.builtins.int;
            Some((of(sema, ty)?, Some(i as u64)))
        }
    }
}

fn struct_layout(sema: &mut Sema, members: &[Member]) -> Option<Layout> {
    let mut bits: u64 = 0;
    let mut align: u32 = 1;

    for &m in members {
        let (layout, width) = member(sema, m)?;
        let unit = u64::from(layout.align) * 8;
        if matches!(m, Member::Symbol(_)) {
            align = align.max(layout.align);
        }
        match width {
            Some(0) => bits = round_up(bits, unit),
            Some(width) => {
                let storage = u64::from(layout.size) * 8;
                if bits % storage + width > storage {
                    bits = round_up(bits, unit);
                }
                bits += width;
            }
            None => {
                bits = round_up(bits, unit);
                bits += u64::from(layout.size) * 8;
            }
        }
    }

    let size = round_up(bits, u64::from(align) * 8) / 8;
    Some(Layout::new(size as u32, align))
}

fn union_layout(sema: &mut Sema, members: &[Member]) -> Option<Layout> {
    let mut size: u64 = 0;
    let mut align: u32 = 1;

    for &m in members {
        let (layout, width) = member(sema, m)?;
        if matches!(m, Member::Symbol(_)) {
            align = align.max(layout.align);
        }
        size = size.max(width.map_or(u64::from(layout.size), |width| width.div_ceil(8)));
    }

    Some(Layout::new(round_up(size, u64::from(align)) as u32, align))
}
