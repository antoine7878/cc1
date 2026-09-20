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

    pub fn run_passes(mut sema: Sema) -> Sema {
        Resolver::resolve_unit(&mut sema);
        eval::check_constants(&mut sema);
        mark_uses(&mut sema);
        finish_externals(&mut sema);
        layout::finalize(&mut sema);
        sema
    }

    pub fn end(sema: Sema) -> &'static Sema {
        semantic::install_sema(sema)
    }

    pub fn analyze(ctx: Context) -> &'static Sema {
        Self::end(Self::run_passes(Self::begin(ctx)))
    }
}
