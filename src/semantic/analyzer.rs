use crate::ast::visit::walk_translation_unit;
use crate::context::Context;
use crate::semantic::{ScopeKind, Sema, SymbolResolver, eval, mark_uses};

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(mut ctx: Context) -> Context {
        let mut sema = Sema::new(ctx.target.clone());
        Self::resolve_names(&mut sema, &ctx);
        eval::check_constants(&mut sema, &ctx);
        mark_uses(&mut sema, &ctx);
        ctx.diagnosis.append(&mut sema.diagnosis);
        ctx.sema = sema;
        ctx
    }

    fn resolve_names(sema: &mut Sema, ctx: &Context) {
        sema.size_expr_facts(ctx.arenas.expressions.len());
        let mut resolver = SymbolResolver::new(sema);
        resolver.sema.scopes.push(ScopeKind::File);
        walk_translation_unit(&mut resolver, ctx, &ctx.ast);
        resolver.sema.scopes.pop();
        assert!(resolver.sema.scopes.is_empty());
    }
}
