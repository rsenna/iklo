//! Backend-agnostic contract tests for any [`Substrate`] implementation.
//!
//! Kept outside `#[cfg(test)]` so other crates (e.g. a future
//! `iklo-substrate-turso`) can reuse [`run_contract_suite`] verbatim against
//! their own backend — see `specs/001-substrate/plan.md` § Contract-test shape.

use crate::{Substrate, Transaction};

/// Runs all seven contract scenarios against a fresh [`Substrate`] built by
/// `make` for each scenario. Drives only the `Substrate`/`Transaction` trait
/// surface — no backend-specific type appears here.
pub fn run_contract_suite<S: Substrate<Value = i64>>(make: impl Fn() -> S) {
    revision_starts_at_zero(&make);
    commit_increments_revision(&make);
    rollback_does_not_increment_revision(&make);
    get_after_set_inside_tx_sees_value(&make);
    get_after_rollback_does_not_see_value(&make);
    get_after_commit_sees_value_from_fresh_tx(&make);
    snapshot_returns_only_committed_state(&make);
}

fn revision_starts_at_zero<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let substrate = make();
    assert_eq!(substrate.revision(), 0);
}

fn commit_increments_revision<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let mut substrate = make();
    let tx = substrate.begin();
    tx.commit().expect("commit");
    assert_eq!(substrate.revision(), 1);
}

fn rollback_does_not_increment_revision<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let mut substrate = make();
    let tx = substrate.begin();
    tx.rollback().expect("rollback");
    assert_eq!(substrate.revision(), 0);
}

fn get_after_set_inside_tx_sees_value<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let mut substrate = make();
    let mut tx = substrate.begin();
    tx.set("x", 42);
    assert_eq!(tx.get("x"), Some(42));
}

fn get_after_rollback_does_not_see_value<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let mut substrate = make();
    let mut tx = substrate.begin();
    tx.set("x", 42);
    tx.rollback().expect("rollback");

    let tx = substrate.begin();
    assert_eq!(tx.get("x"), None);
}

fn get_after_commit_sees_value_from_fresh_tx<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let mut substrate = make();
    let mut tx = substrate.begin();
    tx.set("x", 42);
    tx.commit().expect("commit");

    let tx = substrate.begin();
    assert_eq!(tx.get("x"), Some(42));
}

fn snapshot_returns_only_committed_state<S: Substrate<Value = i64>>(make: &impl Fn() -> S) {
    let mut substrate = make();

    let mut tx = substrate.begin();
    tx.set("x", 1);
    tx.commit().expect("commit");

    // Uncommitted write, then rolled back — must not appear in the snapshot.
    // snapshot() requires no live transaction (enforced by the GAT borrow),
    // so this tx is finalised before we call it.
    let mut tx = substrate.begin();
    tx.set("y", 2);
    tx.rollback().expect("rollback");

    let snapshot = substrate.snapshot();
    assert_eq!(snapshot.get("x"), Some(&1));
    assert_eq!(snapshot.get("y"), None);
}

#[cfg(test)]
mod tests {
    use super::run_contract_suite;
    use crate::memory::InMemorySubstrate;

    #[test]
    fn in_memory_substrate_satisfies_contract() {
        run_contract_suite(InMemorySubstrate::<i64>::new);
    }
}
