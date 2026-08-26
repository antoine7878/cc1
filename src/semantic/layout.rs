use crate::ast::Tag;
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::sema::Sema;
use crate::semantic::{QualifiedType, ResolvedType, SymbolId, TagDefId};
use crate::target::Layout;

fn round_up(value: u64, multiple: u64) -> u64 {
    match value % multiple {
        0 => value,
        rest => value + multiple - rest,
    }
}

pub fn of(sema: &mut Sema, qualified_type: QualifiedType) -> Result<Layout, Diagnosis> {
    if let Some(&layout) = sema.layouts.get(&qualified_type.ty) {
        return Ok(layout);
    }
    let layout = match *sema.types.get(qualified_type.ty) {
        ResolvedType::Tag(id) => of_tag(sema, id)?,
        ResolvedType::Array { elem, len } => {
            let elem = of(sema, elem)?;
            Layout::new(elem.size * len.unwrap_or(0), elem.align)
        }
        ty => sema.target.scalar(&ty).ok_or(Diagnosis::InvalidSizeof)?,
    };
    sema.layouts.insert(qualified_type.ty, layout);
    Ok(layout)
}

pub fn of_tag(sema: &mut Sema, id: TagDefId) -> Result<Layout, Diagnosis> {
    let tag = sema.tags.get(id).clone();
    if !tag.is_complete {
        return Err(Diagnosis::InvalidSizeof);
    }
    match tag.kind {
        Tag::Struct => struct_layout(sema, &tag.members),
        Tag::Union => union_layout(sema, &tag.members),
        Tag::Enum => Ok(sema.target.int),
    }
}

fn member(sema: &mut Sema, id: SymbolId) -> Result<(Layout, Option<u64>), Diagnosis> {
    let symbol = sema.symbols.get(id);
    if !symbol.is_complete {
        return Err(Diagnosis::InvalidSizeof);
    }
    let width = symbol.value.map(|width| width.max(0) as u64);
    let ty = symbol.ty.ok_or(Diagnosis::InvalidSizeof)?;
    Ok((of(sema, ty)?, width))
}

fn struct_layout(sema: &mut Sema, members: &[SymbolId]) -> Result<Layout, Diagnosis> {
    let mut bits: u64 = 0;
    let mut align: u32 = 1;

    for &id in members {
        let (layout, width) = member(sema, id)?;
        let unit = u64::from(layout.align) * 8;
        align = align.max(layout.align);
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
    Ok(Layout::new(size as u32, align))
}

fn union_layout(sema: &mut Sema, members: &[SymbolId]) -> Result<Layout, Diagnosis> {
    let mut size: u64 = 0;
    let mut align: u32 = 1;

    for &id in members {
        let (layout, width) = member(sema, id)?;
        align = align.max(layout.align);
        size = size.max(match width {
            Some(width) => width.div_ceil(8),
            None => u64::from(layout.size),
        });
    }

    Ok(Layout::new(round_up(size, u64::from(align)) as u32, align))
}
