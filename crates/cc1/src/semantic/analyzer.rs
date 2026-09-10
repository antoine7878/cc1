use crate::context::{Context, ctx, install};
use crate::semantic::{self, Sema, SymbolResolver, eval, finish_externals, layout, mark_uses};

pub struct Analyzer;

impl Analyzer {
    pub fn begin(ctx: Context) -> Sema {
        let ctx = install(ctx);
        let mut sema = Sema::new(ctx.target.clone());
        sema.size_tables(&ctx.arenas);
        sema
    }

    pub fn resolve_names(mut sema: Sema) -> Sema {
        SymbolResolver::resolve_unit(&mut sema, ctx());
        sema
    }

    pub fn check_constants(mut sema: Sema) -> Sema {
        eval::check_constants(&mut sema, ctx());
        sema
    }

    pub fn mark_uses(mut sema: Sema) -> Sema {
        mark_uses(&mut sema, ctx());
        sema
    }

    pub fn finish_externals(mut sema: Sema) -> Sema {
        finish_externals(&mut sema);
        sema
    }

    pub fn finalize_layouts(mut sema: Sema) -> Sema {
        layout::finalize(&mut sema);
        sema
    }

    pub fn end(sema: Sema) {
        semantic::install(sema);
    }

    pub fn analyze(ctx: Context) -> &'static Sema {
        let sema = Self::begin(ctx);
        let sema = Self::resolve_names(sema);
        let sema = Self::check_constants(sema);
        let sema = Self::mark_uses(sema);
        let sema = Self::finish_externals(sema);
        let sema = Self::finalize_layouts(sema);
        semantic::install(sema)
    }
}
