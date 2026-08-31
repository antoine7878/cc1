use crate::context::Context;
use crate::utils::{BLUE, RESET};

impl Context {
    pub fn dump_symbols(&self) {
        if self.sema.symbols.is_empty() {
            return;
        }
        println!();
        let rows: Vec<[String; 5]> = self
            .sema
            .symbols
            .data
            .iter()
            .map(|symbol| {
                [
                    symbol.kind.to_string(),
                    symbol.name.id.resolve(self).clone(),
                    symbol.storage.map(|s| s.to_string()).unwrap_or_default(),
                    symbol.value.map(|v| v.to_string()).unwrap_or("-".to_string()),
                    symbol.ty.map(|ty| ty.describe(&self.sema, self)).unwrap_or_default(),
                ]
            })
            .collect();

        print_table(&["KIND", "NAME", "STORAGE", "VALUE", "TYPE"], &rows);
    }

    pub fn dump_diagnostics(&self) {
        if self.diagnosis.is_empty() {
            return;
        }
        for diag in &self.diagnosis {
            println!();
            let _ = diag.print(self);
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
