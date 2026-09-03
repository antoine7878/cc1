use std::collections::HashSet;

use crate::ast::{DeclarationNode, DeclarationSpecifier, InitDeclaratorNode, Name, Storage, StringId};
use crate::context::Context;
use crate::semantic::{DeclaredParams, Diag, ParamInfo, diagnosis::Diagnosis};

pub fn check_external_specifiers(specifiers: &[DeclarationSpecifier]) -> Diag<()> {
    if specifiers
        .iter()
        .any(|s| matches!(s, DeclarationSpecifier::Storage(Storage::Auto | Storage::Register)))
    {
        return Diag::err((), Diagnosis::AutoRegisterExternal);
    }
    Diag::ok(())
}

pub fn check_unique_internal_linkage() -> Diag<()> {
    Diag::ok(())
}

pub fn check_one_external() -> Diag<()> {
    Diag::ok(())
}

pub fn check_function_storage(storage: Storage) -> Diag<()> {
    match storage {
        Storage::Static | Storage::Extern => Diag::ok(()),
        _ => Diag::err((), Diagnosis::FunctionAutoExtern),
    }
}

pub fn extract_function_declarator(params: Option<DeclaredParams>) -> Diag<Option<DeclaredParams>> {
    params.map_or_else(
        || Diag::err(None, Diagnosis::NotFunctionTypeDeclarator),
        |params| Diag::ok(Some(params)),
    )
}

pub fn is_valid_parameter_style(params: &[ParamInfo], old_style_declarations: &[DeclarationNode]) -> Diag<bool> {
    let is_named = params.iter().all(|param| param.name.is_some());
    if !old_style_declarations.is_empty() {
        return Diag::err(is_named, Diagnosis::ParameterTypeListWithList);
    }
    if params.is_empty() {
        return Diag::ok(false);
    }
    match is_named {
        true => Diag::ok(true),
        false => Diag::err(false, Diagnosis::UnnamedPrototypeParameter),
    }
}

pub fn check_void_parameter(is_void: bool) -> Diag<()> {
    match is_void {
        true => Diag::err((), Diagnosis::VoidParameter),
        false => Diag::ok(()),
    }
}

pub fn is_valid_old_style(names: &[StringId], declarations: Vec<Option<StringId>>) -> Diag<Option<Vec<StringId>>> {
    let declarations: HashSet<StringId> = declarations.into_iter().flatten().collect();

    let name_len = names.len();
    let names = names.iter().cloned().collect::<HashSet<_>>();
    if names.len() != name_len {
        return Diag::err(None, Diagnosis::DuplicateParameterName);
    }

    if declarations.difference(&names).next().is_some() {
        return Diag::err(None, Diagnosis::MissingParameterInOldStyle);
    }

    let diff = names.difference(&declarations).cloned().collect::<Vec<_>>();
    Diag::ok(Some(diff))
}

pub fn param_storage_only_register(storage: Storage) -> Diag<Option<()>> {
    match storage {
        Storage::Register => Diag::ok(Some(())),
        _ => Diag::err(None, Diagnosis::ParameterNotRegister),
    }
}

pub fn check_typedef(_ctx: &Context, _name: &Name) -> Diag<bool> {
    Diag::ok(true)
}

pub fn is_tentative_definition(init_declarator: &InitDeclaratorNode, storage: Option<Storage>) -> bool {
    init_declarator.initializer.is_none() && matches!(storage, None | Some(Storage::Static))
}

pub fn tentative_defintion_init_zero() -> Diag<()> {
    Diag::ok(())
}

pub fn no_internal_incomplete_type() -> Diag<bool> {
    Diag::ok(true)
}
