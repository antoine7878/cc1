use crate::ast::StringId;
use crate::ast::visit::walk_translation_unit;
use crate::context::Context;
use crate::semantic::sema::External;
use crate::semantic::{
    Definition, Diag, DiagCollector, Diagnosis, Linkage, QualifiedType, ResolvedType, ScopeKind, Sema, SymbolResolver,
    eval, mark_uses,
};

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(mut ctx: Context) -> Context {
        let mut sema = Sema::new(ctx.target.clone());
        Self::resolve_names(&mut sema, &ctx);
        eval::check_constants(&mut sema, &ctx);
        mark_uses(&mut sema, &ctx);
        finish(&mut sema);
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

fn finish(sema: &mut Sema) {
    let mut entries: Vec<(StringId, External)> =
        sema.externals.iter().map(|(&name, ext)| (name, ext.clone())).collect();
    entries.sort_by_key(|(_, ext)| {
        let p = ext.tentative.or(ext.defined).unwrap_or_default().start;
        (p.file, p.line, p.col)
    });
    for (i, ext) in entries {
        let sym_id = ext.symbol;
        let Some(mut qty) = sema.symbols.get(sym_id).ty else { continue };
        let mut defined = ext.defined;
        if !qty.is_function(sema)
            && defined.is_none()
            && let Some(span) = ext.tentative
        {
            let ty = sema.types.get(qty.id);
            if let ResolvedType::Array { elem, len: None } = ty {
                let ty = sema.types.array(*elem, Some(1));
                qty = QualifiedType::new(ty, qty.is_const, qty.is_volatile);
                sema.symbols.get_mut(sym_id).ty = Some(qty);
            }
            if !qty.is_complete(sema) {
                sema.add_diag(Diag::err((), Diagnosis::TentativeNeverCompleted(qty)), &span);
                continue;
            }
            sema.symbols.get_mut(sym_id).definition = Definition::Definition;
            defined = Some(span);
            if let Some(e) = sema.externals.get_mut(&i) {
                e.defined = Some(span);
            }
        }
        let sym = sema.symbols.get(sym_id);
        if sym.used && defined.is_none() && sym.linkage == Linkage::Internal {
            let span = sym.name.span;
            sema.add_diag(Diag::err((), Diagnosis::InternalNeverDefined(sym.name)), &span);
        }
    }
}
