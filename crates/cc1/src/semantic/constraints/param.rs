use std::collections::HashSet;

use crate::ast::{DeclarationNode, NameId, Storage};
use crate::semantic::diagnostic::{Diag, Diagnostic};
use crate::semantic::{ParamInfo, QualifiedType};

pub fn check_void_param(is_void: bool) -> Diag<()> {
    match is_void {
        true => Diag::err((), Diagnostic::VoidParameter),
        false => Diag::ok(()),
    }
}

pub fn check_declared_void_param(is_tolerated: bool) -> Diag<()> {
    match is_tolerated {
        true => Diag::err((), Diagnostic::NamedVoidParameter),
        false => Diag::err((), Diagnostic::VoidParameter),
    }
}

pub fn check_identifier_list(is_forbidden: bool) -> Diag<()> {
    match is_forbidden {
        true => Diag::err((), Diagnostic::IdentifierListInDeclaration),
        false => Diag::ok(()),
    }
}

pub fn check_complete_param(is_complete: bool, ty: QualifiedType) -> Diag<()> {
    match is_complete {
        true => Diag::ok(()),
        false => Diag::err((), Diagnostic::IncompleteParameter(ty)),
    }
}

pub fn check_param_initializer(has_initializer: bool) -> Diag<()> {
    match has_initializer {
        true => Diag::err((), Diagnostic::InitializedParameter),
        false => Diag::ok(()),
    }
}

pub fn check_declares_parameter(has_declarator: bool) -> Diag<Option<()>> {
    match has_declarator {
        true => Diag::ok(Some(())),
        false => Diag::err(None, Diagnostic::DeclarationWithoutParameter),
    }
}

pub fn check_param_storage(storage: Storage) -> Diag<Option<()>> {
    match storage {
        Storage::Register => Diag::ok(Some(())),
        _ => Diag::err(None, Diagnostic::ParameterNotRegister),
    }
}

pub fn is_valid_param_style(params: &[ParamInfo], old_style_declarations: &[DeclarationNode]) -> Diag<bool> {
    let is_named = params.iter().all(|param| param.name.is_some());
    if !old_style_declarations.is_empty() {
        return Diag::err(is_named, Diagnostic::ParameterTypeListWithList);
    }
    if params.is_empty() {
        return Diag::ok(false);
    }
    match is_named {
        true => Diag::ok(true),
        false => Diag::err(false, Diagnostic::UnnamedPrototypeParameter),
    }
}

pub fn is_valid_old_style(names: &[NameId], declarations: Vec<Option<NameId>>) -> Diag<Option<Vec<NameId>>> {
    let declarations: HashSet<NameId> = declarations.into_iter().flatten().collect();

    let name_len = names.len();
    let names = names.iter().cloned().collect::<HashSet<_>>();
    if names.len() != name_len {
        return Diag::err(None, Diagnostic::DuplicateParameterName);
    }

    if declarations.difference(&names).next().is_some() {
        return Diag::err(None, Diagnostic::MissingParameterInOldStyle);
    }

    let diff = names.difference(&declarations).cloned().collect::<Vec<_>>();
    Diag::ok(Some(diff))
}
