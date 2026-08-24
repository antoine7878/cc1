use crate::parser::Context;
use crate::semantic::resolution::SymbolResolver;
use crate::semantic::{QualifiedType, ResolvedType};
use crate::utils::{BLUE, RESET};

impl SymbolResolver {
    fn describe(&self, ctx: &Context, qt: QualifiedType) -> String {
        let mut out = String::new();
        if qt.is_const {
            out.push_str("const ");
        }
        if qt.is_volatile {
            out.push_str("volatile ");
        }
        match *self.types.get(qt.ty) {
            ResolvedType::Void => out.push_str("void"),
            ResolvedType::Char => out.push_str("char"),
            ResolvedType::SignedChar => out.push_str("signed char"),
            ResolvedType::UnsignedChar => out.push_str("unsigned char"),
            ResolvedType::Short => out.push_str("short"),
            ResolvedType::UnsignedShort => out.push_str("unsigned short"),
            ResolvedType::Int => out.push_str("int"),
            ResolvedType::UnsignedInt => out.push_str("unsigned int"),
            ResolvedType::Long => out.push_str("long"),
            ResolvedType::UnsignedLong => out.push_str("unsigned long"),
            ResolvedType::Float => out.push_str("float"),
            ResolvedType::Double => out.push_str("double"),
            ResolvedType::LongDouble => out.push_str("long double"),
            ResolvedType::Pointer(inner) => {
                out.push_str("pointer to ");
                out.push_str(&self.describe(ctx, inner));
            }
            ResolvedType::Tag(id) => {
                let def = self.tags.get(id);
                let name = def.name.map(|n| n.id.resolve(ctx).as_str()).unwrap_or("<anonymous>");
                out.push_str(&format!("{} {}", def.kind(), name));
                if !def.is_complete {
                    out.push_str(" (incomplete)");
                }
            }
            ResolvedType::Array { elem, len } => {
                out.push_str("array");
                out.push_str(&self.describe(ctx, elem));
                if let Some(len) = len {
                    out.push('[');
                    out.push_str(&len.to_string());
                    out.push('[');
                }
            }
        }
        out
    }

    pub(super) fn report(&self, ctx: &Context) {
        let rows: Vec<[String; 5]> = self
            .symbols
            .data
            .iter()
            .map(|symbol| {
                [
                    symbol.kind.to_string(),
                    symbol.name.id.resolve(ctx).clone(),
                    symbol.storage.map(|s| s.to_string()).unwrap_or_default(),
                    symbol.value.map(|v| v.to_string()).unwrap_or("-".to_string()),
                    symbol.ty.map(|ty| self.describe(ctx, ty)).unwrap_or_default(),
                ]
            })
            .collect();

        println!("Symbols:");
        print_table(&["KIND", "NAME", "STORAGE", "INIT", "TYPE"], &rows);

        println!("Diagnosis:");
        for diag in &self.diagnosis {
            let _ = diag.print(ctx);
        }
    }
}

fn print_table<const N: usize>(headers: &[&str; N], rows: &[[String; N]]) {
    if rows.is_empty() {
        return;
    }

    let mut widths = headers.map(str::len);
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    let rule: [String; N] = std::array::from_fn(|i| "-".repeat(widths[i]));

    println!("{}{}{}", BLUE, table_row(&widths, headers), RESET);
    println!("{}", table_row(&widths, &rule.each_ref().map(String::as_str)));
    for row in rows {
        println!("{}", table_row(&widths, &row.each_ref().map(String::as_str)));
    }
}

fn table_row<const N: usize>(widths: &[usize; N], cells: &[&str; N]) -> String {
    let mut out = String::new();
    for (i, cell) in cells.iter().enumerate() {
        out.push_str(cell);
        if i + 1 < N {
            out.push_str(&" ".repeat(widths[i] - cell.chars().count() + 2));
        }
    }
    out
}
