use crate::ast::visit::walk_translation_unit;
use crate::parser::Context;
use crate::semantic::resolution::SymbolResolver;
use crate::semantic::sema::Sema;
use crate::semantic::{ScopeKind, check};

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        let mut sema = Sema::new(ctx.target);
        Self::resolve_names(&mut sema, &ctx);
        check::run(&mut sema, &ctx);
        sema.into_context(ctx)
    }

    fn resolve_names(sema: &mut Sema, ctx: &Context) {
        let mut resolver = SymbolResolver::new(sema);
        resolver.sema.scopes.push(ScopeKind::File);
        walk_translation_unit(&mut resolver, ctx, &ctx.ast);
        resolver.sema.scopes.pop();
        assert!(resolver.sema.scopes.is_empty());
    }
}
