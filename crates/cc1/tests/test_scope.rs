use cc1::ast::statement::StatementId;
use cc1::ast::{ConstValue, StringId};
use cc1::semantic::{
    Diagnosis, QualifiedType, ResolvedStatement, ResolvedTypeId, ScopeKind, StatementScope, StatementScopes, SymbolId,
    SymbolScopes, TagDefId,
};

fn name(index: usize) -> StringId {
    StringId::from(index)
}

fn symbol(index: usize) -> SymbolId {
    SymbolId::from(index)
}

fn tag(index: usize) -> TagDefId {
    TagDefId::from(index)
}

fn file_scope() -> SymbolScopes {
    let mut scopes = SymbolScopes::default();
    scopes.push(ScopeKind::File);
    scopes
}

#[test]
fn scopes_start_empty() {
    let scopes = SymbolScopes::default();
    assert!(scopes.is_empty());
}

#[test]
fn push_and_pop_balance_out() {
    let mut scopes = file_scope();
    assert!(!scopes.is_empty());
    scopes.push(ScopeKind::Block);
    scopes.pop();
    scopes.pop();
    assert!(scopes.is_empty());
}

#[test]
fn kind_describes_the_innermost_scope() {
    let mut scopes = file_scope();
    assert_eq!(scopes.kind(), ScopeKind::File);
    scopes.push(ScopeKind::Prototype);
    assert_eq!(scopes.kind(), ScopeKind::Prototype);
    scopes.set_kind(ScopeKind::Function);
    assert_eq!(scopes.kind(), ScopeKind::Function);
    scopes.pop();
    assert_eq!(scopes.kind(), ScopeKind::File);
}

#[test]
fn an_undeclared_name_is_not_found() {
    let scopes = file_scope();
    assert_eq!(scopes.lookup_ordinary(name(0)), None);
    assert_eq!(scopes.lookup_tag(name(0), false), None);
}

#[test]
fn lookup_finds_a_name_of_an_enclosing_scope() {
    let mut scopes = file_scope();
    scopes.insert(name(0), symbol(0));
    scopes.push(ScopeKind::Block);
    assert_eq!(scopes.lookup_ordinary(name(0)), Some(symbol(0)));
}

#[test]
fn an_inner_declaration_shadows_an_outer_one() {
    let mut scopes = file_scope();
    scopes.insert(name(0), symbol(0));
    scopes.push(ScopeKind::Block);
    scopes.insert(name(0), symbol(1));
    assert_eq!(scopes.lookup_ordinary(name(0)), Some(symbol(1)));
    scopes.pop();
    assert_eq!(scopes.lookup_ordinary(name(0)), Some(symbol(0)));
}

#[test]
fn a_scope_does_not_outlive_its_pop() {
    let mut scopes = file_scope();
    scopes.push(ScopeKind::Block);
    scopes.insert(name(0), symbol(0));
    scopes.pop();
    assert_eq!(scopes.lookup_ordinary(name(0)), None);
}

#[test]
fn current_only_sees_the_innermost_scope() {
    let mut scopes = file_scope();
    scopes.insert(name(0), symbol(0));
    assert_eq!(scopes.current(name(0)), Some(symbol(0)));
    scopes.push(ScopeKind::Block);
    assert_eq!(scopes.current(name(0)), None);
    assert_eq!(scopes.lookup_ordinary(name(0)), Some(symbol(0)));
}

#[test]
fn ordinary_identifiers_share_one_namespace() {
    let mut scopes = file_scope();
    scopes.insert(name(0), symbol(0));
    assert_eq!(scopes.current(name(0)), Some(symbol(0)));
    assert_eq!(scopes.current(name(0)), Some(symbol(0)));
    assert_eq!(scopes.current(name(0)), Some(symbol(0)));
    assert_eq!(scopes.current(name(0)), Some(symbol(0)));
}

#[test]
fn tags_live_in_their_own_namespace() {
    let mut scopes = file_scope();
    scopes.insert(name(0), symbol(0));
    scopes.insert_tag(name(0), tag(3));
    assert_eq!(scopes.lookup_ordinary(name(0)), Some(symbol(0)));
    assert_eq!(scopes.lookup_tag(name(0), false), Some(tag(3)));
}

#[test]
fn a_tag_is_visible_from_an_inner_scope() {
    let mut scopes = file_scope();
    scopes.insert_tag(name(0), tag(0));
    scopes.push(ScopeKind::Block);
    assert_eq!(scopes.lookup_tag(name(0), false), Some(tag(0)));
    assert_eq!(scopes.lookup_tag(name(0), true), None);
}

#[test]
fn an_inner_tag_shadows_an_outer_one() {
    let mut scopes = file_scope();
    scopes.insert_tag(name(0), tag(0));
    scopes.push(ScopeKind::Block);
    scopes.insert_tag(name(0), tag(1));
    assert_eq!(scopes.lookup_tag(name(0), false), Some(tag(1)));
    assert_eq!(scopes.lookup_tag(name(0), true), Some(tag(1)));
    scopes.pop();
    assert_eq!(scopes.lookup_tag(name(0), false), Some(tag(0)));
}

fn stmt(index: usize) -> StatementId {
    StatementId::from(index)
}

fn control() -> QualifiedType {
    QualifiedType::plain(ResolvedTypeId::from(0))
}

fn switch_scope() -> StatementScopes {
    let mut scopes = StatementScopes::default();
    scopes.push_switch(stmt(0), control());
    scopes
}

#[test]
fn statement_scopes_start_empty() {
    let scopes = StatementScopes::default();
    assert!(scopes.is_empty());
    assert_eq!(scopes.breakable(), None);
    assert_eq!(scopes.nearest_loop(), None);
    assert_eq!(scopes.switch_control(), None);
}

#[test]
fn a_loop_is_both_breakable_and_continuable() {
    let mut scopes = StatementScopes::default();
    scopes.push_loop(stmt(1));
    assert_eq!(scopes.breakable(), Some(stmt(1)));
    assert_eq!(scopes.nearest_loop(), Some(stmt(1)));
    assert_eq!(scopes.switch_control(), None);
}

#[test]
fn a_switch_is_breakable_but_not_continuable() {
    let scopes = switch_scope();
    assert_eq!(scopes.breakable(), Some(stmt(0)));
    assert_eq!(scopes.nearest_loop(), None);
    assert_eq!(scopes.switch_control(), Some(control()));
}

#[test]
fn break_binds_to_the_innermost_scope_and_continue_to_the_innermost_loop() {
    let mut scopes = StatementScopes::default();
    scopes.push_loop(stmt(1));
    scopes.push_switch(stmt(2), control());
    assert_eq!(scopes.breakable(), Some(stmt(2)));
    assert_eq!(scopes.nearest_loop(), Some(stmt(1)));
    scopes.push_loop(stmt(3));
    assert_eq!(scopes.breakable(), Some(stmt(3)));
    assert_eq!(scopes.nearest_loop(), Some(stmt(3)));
}

#[test]
fn a_case_binds_to_the_enclosing_switch_across_a_loop() {
    let mut scopes = switch_scope();
    scopes.push_loop(stmt(1));
    assert_eq!(scopes.switch_control(), Some(control()));
    assert_eq!(scopes.record_case(ConstValue::Int(0), stmt(2)).ok(), Some(stmt(0)));
    assert_eq!(scopes.record_default(stmt(3)).ok(), Some(stmt(0)));
}

#[test]
fn a_case_outside_a_switch_is_rejected() {
    let mut scopes = StatementScopes::default();
    scopes.push_loop(stmt(1));
    assert!(matches!(scopes.record_case(ConstValue::Int(0), stmt(2)), Err(Diagnosis::OutsideSwitch("case"))));
    assert!(matches!(scopes.record_default(stmt(3)), Err(Diagnosis::OutsideSwitch("default"))));
}

#[test]
fn a_case_value_is_unique_within_a_switch() {
    let mut scopes = switch_scope();
    assert_eq!(scopes.record_case(ConstValue::Int(1), stmt(1)).ok(), Some(stmt(0)));
    assert_eq!(scopes.record_case(ConstValue::Int(2), stmt(2)).ok(), Some(stmt(0)));
    assert!(matches!(
        scopes.record_case(ConstValue::Int(1), stmt(3)),
        Err(Diagnosis::DuplicateCase(ConstValue::Int(1)))
    ));
}

#[test]
fn a_nested_switch_owns_its_own_cases() {
    let mut scopes = switch_scope();
    scopes.record_case(ConstValue::Int(1), stmt(1)).unwrap();
    scopes.push_switch(stmt(2), control());
    assert_eq!(scopes.record_case(ConstValue::Int(1), stmt(3)).ok(), Some(stmt(2)));
}

#[test]
fn a_switch_has_at_most_one_default() {
    let mut scopes = switch_scope();
    assert_eq!(scopes.record_default(stmt(1)).ok(), Some(stmt(0)));
    assert!(matches!(scopes.record_default(stmt(2)), Err(Diagnosis::DuplicateDefault)));
}

#[test]
fn leaving_a_scope_yields_the_statement_it_resolves() {
    let mut scopes = StatementScopes::default();
    scopes.push_loop(stmt(1));
    let scope = scopes.pop();
    assert!(matches!(scope, Some(StatementScope::Loop(id)) if id == stmt(1)));
    let (id, resolved) = scope.unwrap().into_resolved();
    assert_eq!(id, stmt(1));
    assert!(resolved.is_none());
    assert!(scopes.is_empty());
    assert!(scopes.pop().is_none());
}

#[test]
fn leaving_a_switch_carries_its_cases_out() {
    let mut scopes = switch_scope();
    scopes.record_case(ConstValue::Int(7), stmt(1)).unwrap();
    scopes.record_default(stmt(2)).unwrap();
    let Some(scope) = scopes.pop() else { panic!("a switch statement") };
    let (id, resolved) = scope.into_resolved();
    let Some(ResolvedStatement::Switch { control, cases, default }) = resolved else {
        panic!("a switch statement")
    };
    assert_eq!(id, stmt(0));
    assert_eq!(control, self::control());
    assert_eq!(cases, vec![(ConstValue::Int(7), stmt(1))]);
    assert_eq!(default, Some(stmt(2)));
    assert!(scopes.is_empty());
}
