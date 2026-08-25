use cc1::arena::{Arena, ArenaId, Interner};

type Id = ArenaId<String>;

fn arena() -> Arena<Id, String> {
    Arena::default()
}

fn interner() -> Interner<Id, String> {
    Interner::default()
}

#[test]
fn arena_starts_empty() {
    let arena = arena();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_hands_out_sequential_ids() {
    let mut arena = arena();
    let first = arena.alloc("a".to_string());
    let second = arena.alloc("b".to_string());
    assert_eq!(usize::from(first), 0);
    assert_eq!(usize::from(second), 1);
    assert_ne!(first, second);
    assert_eq!(arena.len(), 2);
    assert!(!arena.is_empty());
}

#[test]
fn arena_resolves_an_id_to_its_value() {
    let mut arena = arena();
    let first = arena.alloc("a".to_string());
    let second = arena.alloc("b".to_string());
    assert_eq!(arena.get(first), "a");
    assert_eq!(arena.get(second), "b");
}

#[test]
fn arena_keeps_duplicates_distinct() {
    let mut arena = arena();
    let first = arena.alloc("a".to_string());
    let second = arena.alloc("a".to_string());
    assert_ne!(first, second);
    assert_eq!(arena.len(), 2);
}

#[test]
fn arena_get_mut_updates_in_place() {
    let mut arena = arena();
    let id = arena.alloc("a".to_string());
    arena.get_mut(id).push('b');
    assert_eq!(arena.get(id), "ab");
    assert_eq!(arena.len(), 1);
}

#[test]
fn arena_exposes_its_values_in_allocation_order() {
    let mut arena = arena();
    arena.alloc("a".to_string());
    arena.alloc("b".to_string());
    assert_eq!(arena.data, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn interner_starts_empty() {
    let interner = interner();
    assert!(interner.is_empty());
    assert_eq!(interner.len(), 0);
}

#[test]
fn interner_gives_one_id_per_distinct_value() {
    let mut interner = interner();
    let first = interner.alloc("a".to_string());
    let second = interner.alloc("b".to_string());
    assert_ne!(first, second);
    assert_eq!(interner.len(), 2);
}

#[test]
fn interner_reuses_the_id_of_an_equal_value() {
    let mut interner = interner();
    let first = interner.alloc("a".to_string());
    let other = interner.alloc("b".to_string());
    let again = interner.alloc("a".to_string());
    assert_eq!(first, again);
    assert_ne!(first, other);
    assert_eq!(interner.len(), 2);
}

#[test]
fn interner_resolves_an_id_to_its_value() {
    let mut interner = interner();
    let first = interner.alloc("a".to_string());
    let second = interner.alloc("b".to_string());
    assert_eq!(interner.get(first), "a");
    assert_eq!(interner.get(second), "b");
    let again = interner.alloc("a".to_string());
    assert_eq!(interner.get(again), "a");
}

#[test]
fn ids_are_compared_by_index() {
    let mut arena = arena();
    let id = arena.alloc("a".to_string());
    assert_eq!(id, Id::from(0usize));
    assert_ne!(id, Id::from(1usize));
    assert_eq!(usize::from(id), 0);
}

#[test]
fn ids_are_named_after_their_value_type() {
    let id = Id::from(7usize);
    assert_eq!(format!("{id}"), "StringId(7)");
    assert_eq!(format!("{id:?}"), "StringId(7)");
}
