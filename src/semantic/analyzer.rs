use crate::arena::ResolveMutWith;
use crate::ast::visit::walk_translation_unit;
use crate::context::Context;
use crate::semantic::{
    Diagnosis, Linkage, QualifiedType, ResolvedType, ScopeKind, Sema, SymbolResolver, eval, mark_uses,
};

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

fn finish(sema: &mut Sema) -> Result<(), Diagnosis> {
    for ext in sema.externals.values() {
        let sym_id = ext.symbol;
        let sym = sema.symbols.get(sym_id);
        let Some(qty) = sym.ty else { continue };
        // if Object and defined_at is unset and tentative_at is set:
        if qty.is_object(sema) && ext.defined.is_none() && ext.tentative.is_some() {
            let ty = sema.types.get(qty.id);
            // if type is an array of unknown size -> complete it with one element
            if let ResolvedType::Array { elem, len: None } = ty {
                let ty = sema.types.array(*elem, Some(1));
                sema.symbols.get_mut(sym_id).ty = Some(QualifiedType::new(ty, false, false));
            }
            let ty = sema.types.get(qty.id);
            // if type is still incomplete -> error TentativeNeverCompleted(name, type)
            if !ty.is_complete(sema) {
                return Err(Diagnosis::DivisionByZero);
            }
            // else mark defined, with an implicit all-zero initialiser
        }
        let sym = sema.symbols.get(sym_id);
        // if used and defined_at is unset and linkage is Internal:
        if sym.used && ext.defined.is_none() && sym.linkage == Linkage::Internal {
            // error InternalNeverDefined(kind, name)
            return Err(Diagnosis::DivisionByZero);
        }
    }
    Ok(())
}

// finish_unit():
//     for each entry:
//         if Object and defined_at is unset and tentative_at is set:
//             if type is an array of unknown size -> complete it with one element
//             if type is still incomplete -> error TentativeNeverCompleted(name, type)
//             else mark defined, with an implicit all-zero initialiser
//         if used and defined_at is unset and linkage is Internal:
//             error InternalNeverDefined(kind, name)
