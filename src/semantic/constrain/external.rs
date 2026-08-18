use crate::ast::{DeclarationSpecifier, Storage};
use crate::semantic::{Diag, diagnosis::DiagnosisInner};

/// 6.7 External definitions
/// The storage-class specifiers auto and register shall not appear in the declaration specifiers in an external declaration.
pub fn check_external_specifiers(specifiers: &[DeclarationSpecifier]) -> Diag<()> {
    if specifiers
        .iter()
        .any(|s| matches!(s, DeclarationSpecifier::Storage(Storage::Auto | Storage::Register)))
    {
        return Diag::with_diag((), DiagnosisInner::AutoRegisterExternal);
    }
    Diag::res(())
}

/// 6.7 External definitions
/// There shall be no more than one external definition for each identifier declared with internal
/// linkage in a translation unit.
pub fn check_unique_internal_linkage() -> Diag<()> {
    Diag::res(())
}

/// 6.7 External definitions
/// If an identifier declared with internal linkage is used in an expression
/// (other than as a part of the operand of a sizeof operator), there shall be exactly
/// one external definition for the identifier in the translation unit.
pub fn check_one_external() -> Diag<()> {
    Diag::res(())
}
