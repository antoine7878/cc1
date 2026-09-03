use crate::context::Context;
use crate::utils::print_table;

impl Context {
    pub fn dump_symbols(&self) {
        if self.sema.symbols.is_empty() {
            return;
        }
        println!();
        let rows: Vec<[String; 8]> = self
            .sema
            .symbols
            .iter()
            .map(|symbol| {
                [
                    symbol.kind.to_string(),
                    symbol.name.id.resolve(self).clone(),
                    symbol.storage.map_or(String::default(), |s| s.to_string()),
                    symbol.linkage.to_string(),
                    symbol.duration.to_string(),
                    symbol.definition.to_string(),
                    symbol.value.map_or("-".to_string(), |v| v.to_string()),
                    symbol.ty.map_or(String::default(), |ty| ty.describe(&self.sema, self).to_string()),
                ]
            })
            .collect();

        print_table(
            &["KIND", "NAME", "STORAGE", "LINKAGE", "DURATION", "DEFINITION", "VALUE", "TYPE"],
            &rows,
        );
    }

    pub fn dump_diagnostics(&self) {
        if self.diagnosis.is_empty() {
            return;
        }
        for diag in &self.diagnosis {
            let _ = diag.print(self);
        }
    }
}
