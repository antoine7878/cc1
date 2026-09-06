use crate::ast::visit::walk_translation_unit;
use crate::context::Context;
use crate::semantic::{ScopeKind, Sema, SymbolResolver, eval, finish_externals, mark_uses};

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let ctx = Self::init(ctx);
        let ctx = Self::resolve_names(ctx);
        let ctx = Self::check_constants(ctx);
        let ctx = Self::mark_uses(ctx);
        Self::finish(ctx)
    }

    pub fn init(mut ctx: Context) -> Context {
        let mut sema = Sema::new(ctx.target.clone());
        sema.size_tables(&ctx.arenas);
        ctx.sema = sema;
        ctx
    }

    pub fn resolve_names(ctx: Context) -> Context {
        Sema::with_sema(ctx, |sema, ctx| {
            let mut resolver = SymbolResolver::new(sema);
            resolver.sema.scopes.push(ScopeKind::File);
            walk_translation_unit(&mut resolver, ctx, &ctx.ast);
            resolver.sema.scopes.pop();
            assert!(resolver.sema.scopes.is_empty());
        })
    }

    pub fn check_constants(ctx: Context) -> Context {
        Sema::with_sema(ctx, eval::check_constants)
    }

    pub fn mark_uses(ctx: Context) -> Context {
        Sema::with_sema(ctx, mark_uses)
    }

    pub fn finish(ctx: Context) -> Context {
        Sema::with_sema(ctx, |sema, _| finish_externals(sema))
    }
}
