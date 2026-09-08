use crate::context::Context;
use crate::semantic::{Sema, SymbolResolver, eval, finish_externals, layout, mark_uses};

pub struct Analyzer;

impl Analyzer {
    pub fn init(mut ctx: Context) -> Context {
        let mut sema = Sema::new(ctx.target.clone());
        sema.size_tables(&ctx.arenas);
        ctx.sema = sema;
        ctx
    }

    pub fn resolve_names(ctx: Context) -> Context {
        Sema::with_sema(ctx, SymbolResolver::resolve_unit)
    }

    pub fn check_constants(ctx: Context) -> Context {
        Sema::with_sema(ctx, eval::check_constants)
    }

    pub fn mark_uses(ctx: Context) -> Context {
        Sema::with_sema(ctx, mark_uses)
    }

    pub fn finish_externals(ctx: Context) -> Context {
        Sema::with_sema(ctx, |sema, _| finish_externals(sema))
    }

    pub fn finalize_layouts(ctx: Context) -> Context {
        Sema::with_sema(ctx, |sema, _| layout::finalize(sema))
    }

    pub fn analyze(ctx: Context) -> Context {
        let ctx = Self::init(ctx);
        let ctx = Self::resolve_names(ctx);
        let ctx = Self::check_constants(ctx);
        let ctx = Self::mark_uses(ctx);
        let ctx = Self::finish_externals(ctx);
        Self::finalize_layouts(ctx)
    }
}
