use cc1::ast::StringId;
use cc1::semantic::{ScopeKind, SymbolId, SymbolScopes, TagDefId};

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
