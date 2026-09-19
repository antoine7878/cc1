use crate::ast::NameId;
use crate::semantic::sema::External;
use crate::semantic::{DefinitionState, Diag, Diagnostic, DiagnosticSink, Linkage, QualifiedType, ResolvedType, Sema};

pub fn finish_externals(sema: &mut Sema) {
    let mut entries: Vec<(NameId, External)> = sema.externals.iter().map(|(&name, ext)| (name, ext.clone())).collect();
    entries.sort_by_key(|(_, ext)| {
        let p = ext.tentative.or(ext.defined).unwrap_or_default().start;
        (p.file, p.line, p.col)
    });
    for (i, ext) in entries {
        let sym_id = ext.symbol;
        let mut qty = sema.symbols.get(sym_id).ty;
        let mut defined = ext.defined;
        if !qty.is_function(sema)
            && defined.is_none()
            && let Some(span) = ext.tentative
        {
            let ty = sema.types.get(qty.id);
            if let ResolvedType::Array { elem, len: None } = ty {
                let ty = sema.types.array(*elem, Some(1));
                qty = QualifiedType::new(ty, qty.is_const, qty.is_volatile);
                sema.symbols.get_mut(sym_id).ty = qty;
            }
            if !qty.is_complete(sema) {
                sema.add_diag(Diag::err((), Diagnostic::TentativeNeverCompleted(qty)), &span);
                continue;
            }
            sema.symbols.get_mut(sym_id).definition = DefinitionState::Defined;
            defined = Some(span);
            if let Some(e) = sema.externals.get_mut(&i) {
                e.defined = Some(span);
            }
        }
        let sym = sema.symbols.get(sym_id);
        if sym.used && defined.is_none() && sym.linkage == Linkage::Internal {
            let span = sym.name.span;
            sema.add_diag(Diag::err((), Diagnostic::InternalNeverDefined(sym.name)), &span);
        }
    }
}
