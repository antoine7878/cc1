use std::collections::HashSet;

use crate::ast::{DeclarationNode, Storage, StringId};
use crate::semantic::diagnosis::{Diag, Diagnosis};
use crate::semantic::{ParamInfo, QualifiedType};

pub fn check_void_parameter(is_void: bool) -> Diag<()> {
    match is_void {
        true => Diag::err((), Diagnosis::VoidParameter),
        false => Diag::ok(()),
    }
}

pub fn check_complete_parameter(is_complete: bool, ty: QualifiedType) -> Diag<()> {
    match is_complete {
        true => Diag::ok(()),
        false => Diag::err((), Diagnosis::IncompleteParameter(ty)),
    }
}

pub fn param_storage_only_register(storage: Storage) -> Diag<Option<()>> {
    match storage {
        Storage::Register => Diag::ok(Some(())),
        _ => Diag::err(None, Diagnosis::ParameterNotRegister),
    }
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
