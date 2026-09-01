use crate::utils::{BLUE, RESET};

pub fn print_table<const N: usize>(headers: &[&str; N], rows: &[[String; N]]) {
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
