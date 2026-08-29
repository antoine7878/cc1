use std::collections::HashSet;

use crate::ast::{DeclarationNode, DeclarationSpecifier, InitDeclaratorNode, Name, Storage, StringId};
use crate::parser::Context;
use crate::semantic::{DeclaredParams, Diag, ParamInfo, diagnosis::Diagnosis};

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
/// a function type, as specifed by the declarator portion of the function definition.
pub fn extract_function_declarator(params: Option<DeclaredParams>) -> Diag<Option<DeclaredParams>> {
    match params {
        Some(params) => Diag::some(params),
        None => Diag::none_diag(Diagnosis::NotFunctionTypeDeclarator),
    }
}

/// 6.7.1 Function definitions
/// If the declarator includes a parameter type list. the declaration of each parameter shall include
/// an identifier (except for the special case of a parameter list consisting of a single parameter of
/// type void, in which there shall not be an identifier). No declaration list shall follow.
/// return value:
///     false -> valid function no parameter symbol
///     true -> valid function with parameter symbol
///     Diagnosis -> invalid function
pub fn is_valid_parameter_style(params: &[ParamInfo], old_style_declarations: &[DeclarationNode]) -> Diag<bool> {
    let is_named = params.iter().all(|param| param.name.is_some());
    if !old_style_declarations.is_empty() {
        return Diag::with_diag(is_named, Diagnosis::ParameterTypeListWithList);
    }
    if params.is_empty() {
        return Diag::res(false);
    }
    match is_named {
        true => Diag::res(true),
        false => Diag::with_diag(false, Diagnosis::AbstractParameterDeclaration),
    }
}

/// 6.7.1 Function definitions
/// The resulting parameter type shall be an object type.
pub fn check_void_parameter(is_void: bool) -> Diag<()> {
    match is_void {
        true => Diag::only_diag(Diagnosis::VoidParameter),
        false => Diag::res(()),
    }
}

/// 6.7.1 Function definitions
/// If the declarator includes an identifier list, the types of the parameters may be declared in a following declaration list,
/// any parameter that is not declared has type int.
pub fn is_valid_old_style(names: &[StringId], declarations: Vec<Option<StringId>>) -> Diag<Option<Vec<StringId>>> {
    let declarations_len = declarations.len();
    let Some(declarations) = declarations.into_iter().collect::<Option<HashSet<StringId>>>() else {
        return Diag::res(None);
    };

    let name_len = names.len();
    let names = names.iter().cloned().collect::<HashSet<_>>();
    if names.len() != name_len {
        return Diag::with_diag(None, Diagnosis::DuplicateParameterName);
    }

    if declarations.difference(&names).next().is_some() {
        return Diag::with_diag(None, Diagnosis::MissingParameterInOldStyle);
    }

    let diff = names.difference(&declarations).cloned().collect::<Vec<_>>();
    Diag::res(Some(diff))
}

/// 6.7.1 Function definitions
/// The declarations in the declaration list shall contain no storage-class specifier other than register and, no initializations.
pub fn param_storage_only_register(storage: Storage) -> Diag<Option<()>> {
    match storage {
        Storage::Register => Diag::some(()),
        _ => Diag::none_diag(Diagnosis::ParameterNotRegister),
    }
}

/// 6.7.1 Function definitions
/// [In old style] An identifier declared as a typedef name shall not be redeclared as a parameter.
pub fn check_typedef(_ctx: &Context, _name: &Name) -> Diag<bool> {
    Diag::res(true)
}

/// 6.7.2 External object definitions
/// A declaration of an identifier for an object that has file scope without an initializer and
/// without a storage-clash specitier or with the storage-class specifier static,
/// constitutes a tentative definition
pub fn is_tentative_definition(init_declarator: &InitDeclaratorNode, storage: Option<Storage>) -> bool {
    init_declarator.initializer.is_none() && matches!(storage, None | Some(Storage::Static))
}

/// 6.7.2 External object definitions
/// If a translation unit contains one or more tentative definitions for an
/// identifier. and the translation unit contains no external definition for that identifier. then the
/// behavior is exacti! ah it‘ the trun&tion unit contains a file scope declaration of that identifier.
/// with the composite type air of the end of the translation unit. with an initializer equal to 0
pub fn tentative_defintion_init_zero() -> Diag<()> {
    Diag::res(())
}

/// 6.7.2 External object definitions
/// If the declaration of an identifier for an object is a tentative definition and has internal linkage,
/// the declared type shall not be an incomplete type.
pub fn no_internal_incomplete_type() -> Diag<bool> {
    Diag::res(true)
}
