use cc1::arena::{ArenaId, HasTable, Loan, SideTable};

type Id = ArenaId<String>;

fn table() -> SideTable<Id, String> {
    SideTable::default()
}

struct Holder {
    table: SideTable<Id, String>,
}

impl HasTable<Id, String> for Holder {
    fn table(&mut self) -> &mut SideTable<Id, String> {
        &mut self.table
    }
}

fn holder() -> Holder {
    let mut table = table();
    table.resize(3);
    Holder { table }
}

type Ops<'h, const N: usize> = Loan<'h, Holder, Id, String, N>;

#[test]
fn default_table_is_empty_and_reads_unknown() {
    let table = table();
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    let id = Id::from(0usize);
    assert!(!table.seen(id));
    assert!(!table.poisoned(id));
    assert_eq!(table.get(id), None);
}

#[test]
fn resize_grows_and_fills_with_unknown() {
    let mut table = table();
    table.resize(3);
    assert_eq!(table.len(), 3);
    for i in 0..3 {
        let id = Id::from(i);
        assert!(!table.seen(id));
        assert!(!table.poisoned(id));
    }
}

#[test]
fn resize_does_not_shrink_and_keeps_stored_values() {
    let mut table = table();
    table.resize(3);
    let id = Id::from(1usize);
    table.set(id, Some("a".to_string()));
    table.resize(1);
    assert_eq!(table.len(), 3);
    assert_eq!(table.get(id), Some(&"a".to_string()));
}

#[test]
fn set_some_makes_it_known() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, Some("a".to_string()));
    assert!(table.seen(id));
    assert!(!table.poisoned(id));
    assert_eq!(table.get(id), Some(&"a".to_string()));
}

#[test]
fn set_none_makes_it_poisoned() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, None);
    assert!(table.seen(id));
    assert!(table.poisoned(id));
    assert_eq!(table.get(id), None);
}

#[test]
fn set_overwrites_known_with_poisoned() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, Some("a".to_string()));
    table.set(id, None);
    assert!(table.poisoned(id));
    assert_eq!(table.get(id), None);
}

#[test]
fn set_overwrites_poisoned_with_known() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, None);
    table.set(id, Some("a".to_string()));
    assert!(!table.poisoned(id));
    assert_eq!(table.get(id), Some(&"a".to_string()));
}

#[test]
fn get_mut_mutates_in_place() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, Some("a".to_string()));
    table.get_mut(id).unwrap().push('b');
    assert_eq!(table.get(id), Some(&"ab".to_string()));
}

#[test]
fn take_known_yields_value_and_leaves_borrowed_state() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, Some("a".to_string()));
    assert_eq!(table.take(id), Some("a".to_string()));
    assert!(table.seen(id));
    assert!(!table.poisoned(id));
    assert_eq!(table.get(id), None);
}

#[test]
fn take_twice_returns_none_the_second_time() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, Some("a".to_string()));
    table.take(id);
    assert_eq!(table.take(id), None);
}

#[test]
fn take_on_unknown_returns_none_and_leaves_state() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    assert_eq!(table.take(id), None);
    assert!(!table.seen(id));
}

#[test]
fn take_on_poisoned_returns_none_and_leaves_state() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, None);
    assert_eq!(table.take(id), None);
    assert!(table.poisoned(id));
}

#[test]
fn give_after_take_restores_known() {
    let mut table = table();
    table.resize(1);
    let id = Id::from(0usize);
    table.set(id, Some("a".to_string()));
    let value = table.take(id).unwrap();
    table.give(id, value);
    assert!(!table.poisoned(id));
    assert_eq!(table.get(id), Some(&"a".to_string()));
}

#[test]
fn reads_past_the_end_do_not_panic() {
    let table = table();
    let id = Id::from(42usize);
    assert!(!table.seen(id));
    assert!(!table.poisoned(id));
    assert_eq!(table.get(id), None);
}

#[test]
fn loan_take_yields_both_values_and_drop_restores_known() {
    let mut holder = holder();
    let id0 = Id::from(0usize);
    let id1 = Id::from(1usize);
    holder.table.set(id0, Some("a".to_string()));
    holder.table.set(id1, Some("b".to_string()));
    {
        let mut ops: Ops<2> = Loan::take(&mut holder, [id0, id1]).unwrap();
        let (_, [v0, v1]) = ops.parts();
        assert_eq!(v0, "a");
        assert_eq!(v1, "b");
    }
    assert_eq!(holder.table.get(id0), Some(&"a".to_string()));
    assert_eq!(holder.table.get(id1), Some(&"b".to_string()));
}

#[test]
fn loan_mutation_through_parts_is_visible_after_drop() {
    let mut holder = holder();
    let id0 = Id::from(0usize);
    holder.table.set(id0, Some("a".to_string()));
    {
        let mut ops: Ops<1> = Loan::take(&mut holder, [id0]).unwrap();
        let (_, [v0]) = ops.parts();
        v0.push('b');
    }
    assert_eq!(holder.table.get(id0), Some(&"ab".to_string()));
}

#[test]
fn loan_fails_when_second_id_poisoned_and_restores_first_to_known() {
    let mut holder = holder();
    let id0 = Id::from(0usize);
    let id1 = Id::from(1usize);
    holder.table.set(id0, Some("a".to_string()));
    holder.table.set(id1, None);
    let loan: Option<Ops<2>> = Loan::take(&mut holder, [id0, id1]);
    assert!(loan.is_none());
    drop(loan);
    assert_eq!(holder.table.get(id0), Some(&"a".to_string()));
    assert!(!holder.table.poisoned(id0));
    assert!(holder.table.poisoned(id1));
}

#[test]
fn loan_fails_when_id_unknown_and_changes_nothing() {
    let mut holder = holder();
    let id0 = Id::from(2usize);
    let loan: Option<Ops<1>> = Loan::take(&mut holder, [id0]);
    assert!(loan.is_none());
    drop(loan);
    assert!(!holder.table.seen(id0));
}

#[test]
fn loaned_id_reads_as_none_but_seen_while_loan_is_alive() {
    let mut holder = holder();
    let id0 = Id::from(0usize);
    holder.table.set(id0, Some("a".to_string()));
    let mut ops: Ops<1> = Loan::take(&mut holder, [id0]).unwrap();
    let (holder, _) = ops.parts();
    assert_eq!(holder.table.get(id0), None);
    assert!(holder.table.seen(id0));
}
