use libft::print_table;

use crate::context::ctx;
use crate::semantic::{DiagnosticNode, sema};

pub fn dump_symbols() {
    let sema = sema();
    if sema.symbols.is_empty() {
        return;
    }
    println!();
    let rows: Vec<[String; 9]> = sema
        .symbols
        .iter()
        .map(|symbol| {
            [
                symbol.kind.to_string(),
                symbol.name.id.resolve().clone(),
                symbol.storage.map_or(String::default(), |s| s.to_string()),
                symbol.linkage.to_string(),
                symbol.duration.to_string(),
                symbol.definition.to_string(),
                symbol.value.map_or("-".to_string(), |v| v.to_string()),
                symbol.ty.display(sema).to_string(),
                symbol.used.to_string(),
            ]
        })
        .collect();

    print_table(&["KIND", "NAME", "STORAGE", "LINKAGE", "DURATION", "DEFINITION", "VALUE", "TYPE", "USED"], &rows);
}

pub fn dump_diagnostics(codegen: &Vec<DiagnosticNode>) {
    for diag in ctx().diagnostics.iter().chain(&sema().diagnostics).chain(codegen) {
        let _ = diag.print();
    }
}
