use crate::context::{Context, install_context};
use crate::semantic::{self, Resolver, Sema, eval, finish_externals, layout, mark_uses};

pub struct Analyzer;

impl Analyzer {
    pub fn begin(ctx: Context) -> Sema {
        let ctx = install_context(ctx);
        let mut sema = Sema::default();
        sema.size_tables(&ctx.arenas);
        sema
    }

    pub fn resolve_names(mut sema: Sema) -> Sema {
        Resolver::resolve_unit(&mut sema);
        sema
    }

    pub fn check_constants(mut sema: Sema) -> Sema {
        eval::check_constants(&mut sema);
        sema
    }

    pub fn mark_uses(mut sema: Sema) -> Sema {
        mark_uses(&mut sema);
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
        semantic::install_sema(sema);
    }

    pub fn analyze(ctx: Context) -> &'static Sema {
        let sema = Self::begin(ctx);
        let sema = Self::resolve_names(sema);
        let sema = Self::check_constants(sema);
        let sema = Self::mark_uses(sema);
        let sema = Self::finish_externals(sema);
        let sema = Self::finalize_layouts(sema);
        semantic::install_sema(sema)
    }
}
