use crate::ast::{
    DeclarationSpecifier, Declarator, DeclaratorNode, FunctionParameters, FunctionParametersNode, ParameterDeclaration,
    Storage,
};
use crate::semantic::{Diag, diagnosis::Diagnosis};

/// 6.7 External definitions
/// The storage-class specifiers auto and register shall not appear in the declaration specifiers in an external declaration.
pub fn check_external_specifiers(specifiers: &[DeclarationSpecifier]) -> Diag<()> {
    if specifiers
        .iter()
        .any(|s| matches!(s, DeclarationSpecifier::Storage(Storage::Auto | Storage::Register)))
    {
        return Diag::with_diag((), Diagnosis::AutoRegisterExternal);
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

/// 6.7.1 Function definitions
/// The storage-class specifier, if any. in the declaration specifiers shall be either extern or static
pub fn check_function_storage(storage: Storage) -> Diag<()> {
    match storage {
        Storage::Static | Storage::Extern => Diag::res(()),
        _ => Diag::with_diag((), Diagnosis::FunctionAutoExtern),
    }
}

/// 6.7.1 Function definitions
/// The identifier declared in a function definition (which is the name of the function) shall have
/// a function type. as specifed by the declarator portion of the function definition.
pub fn extract_function_declarator(
    declarator: &Declarator,
) -> Diag<Option<(&DeclaratorNode, &FunctionParametersNode)>> {
    match declarator {
        Declarator::Function { declarator, params } => Diag::res(Some((declarator, params))),
        _ => Diag::with_diag(None, Diagnosis::NotFunctionTypeDeclarator),
    }
}

/// 6.7.1 Function definitions
/// If the declarator includes a parameter type list. the declaration of each parameter shall include
/// an identifier (except for the special case of a parameter list consisting of a single parameter of
/// type void, in which there shall not be an identifier). No declaration list shall follow.
pub fn _get2(declarator: &Declarator, params: &[ParameterDeclaration]) -> Diag<bool> {
    Diag::res(true)
}

pub fn _get(declarator: &Declarator, params: &FunctionParametersNode) -> Diag<bool> {
    match &params.param {
        FunctionParameters::Empty => Diag::res(true),
        FunctionParameters::ParameterTypeList(_a) | FunctionParameters::Variadic(_a) => {}
        FunctionParameters::OldStyle(_a) => Diag::res(true),
    }
    // case void
    // case all ident
    // case no ident
    // Diag::res(())
}
