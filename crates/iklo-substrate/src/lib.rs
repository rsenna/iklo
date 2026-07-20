//! Capability boundary that hides *where* the runtime image lives.
//!
//! See `specs/001-substrate/spec.md` and `spec/decisions/ADR-0001-substrate-boundary.md`.

use std::collections::HashMap;
use std::fmt;

pub mod contract;

/// Errors returned by [`Transaction::commit`] and [`Transaction::rollback`].
///
/// Intentionally minimal today — future backends (e.g. Turso) will extend
/// this with I/O and persistence errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstrateError {
    /// A binding operation failed (details depend on the backend).
    BindingFailed(String),
}

impl fmt::Display for SubstrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubstrateError::BindingFailed(msg) => write!(f, "binding failed: {msg}"),
        }
    }
}

impl std::error::Error for SubstrateError {}

/// The runtime image backend.
///
/// A `Substrate` owns the authoritative binding state and revision counter.
/// Callers access state exclusively through [`Transaction`]s obtained via
/// [`begin`](Substrate::begin).
///
/// # Transaction contract
///
/// 1. [`begin`](Substrate::begin) borrows the substrate **mutably** — while a
///    transaction is alive, no second `begin`, `snapshot`, or `revision` call
///    compiles.  This is enforced at compile time by the lifetime on `Tx<'a>`.
/// 2. The transaction holds a *working copy* of the bindings.  Reads ([`get`](Transaction::get))
///    see uncommitted writes; the authoritative state is untouched until
///    [`commit`](Transaction::commit).
/// 3. [`commit`](Transaction::commit) consumes the transaction and writes the
///    working copy back, incrementing the revision counter.
/// 4. [`rollback`](Transaction::rollback) consumes the transaction and drops the
///    working copy — the substrate is unchanged.
/// 5. Both `commit` and `rollback` consume `self`, so the same transaction
///    cannot be finalised twice.  (If `Tx` implements `Clone`, this guarantee
///    no longer holds at compile time — implementations should avoid that.)
///
/// # Revision semantics
///
/// The revision counter starts at zero and increments on each successful
/// [`commit`](Transaction::commit).  [`rollback`](Transaction::rollback) does
/// **not** increment it.
pub trait Substrate {
    /// The value type stored in bindings.
    type Value: Clone + fmt::Debug;

    /// A transaction borrows the substrate for its entire lifetime.
    ///
    /// The GAT (`type Tx<'a>`) ties the transaction's lifetime to `&mut self`,
    /// preventing a second `begin` / `snapshot` / `revision` call while a
    /// transaction is live.
    type Tx<'a>: Transaction<Value = Self::Value>
    where
        Self: 'a;

    /// Open a new transaction.
    ///
    /// While the returned [`Tx`](Substrate::Tx) is alive, the substrate is
    /// mutably borrowed — no other transaction or snapshot can be taken.
    fn begin(&mut self) -> Self::Tx<'_>;

    /// The number of successfully committed transactions.
    fn revision(&self) -> u64;

    /// An owned snapshot of all committed bindings.
    ///
    /// Returns the state as of the last commit, **not** any uncommitted
    /// transactional writes.  The owned `HashMap` avoids leaking
    /// backend-specific storage representations.
    fn snapshot(&self) -> HashMap<String, Self::Value>;
}

/// A transactional view of binding state.
///
/// Obtained from [`Substrate::begin`].  Holds a working copy of the bindings;
/// reads see uncommitted writes.  Consumed by [`commit`](Transaction::commit)
/// or [`rollback`](Transaction::rollback) — both consume `self`, so only one
/// finalisation is possible per transaction value.
pub trait Transaction {
    /// The value type stored in bindings.
    type Value: Clone;

    /// Read a binding by name from the working copy.
    ///
    /// Returns `None` if the name is not bound in this transaction.
    fn get(&self, name: &str) -> Option<Self::Value>;

    /// Write a binding into the working copy.
    ///
    /// Overwrites any previous value for `name`.  The substrate's authoritative
    /// state is unchanged until [`commit`](Transaction::commit).
    fn set(&mut self, name: &str, value: Self::Value);

    /// Finalise the transaction, writing the working copy back to the
    /// substrate and incrementing the revision counter.
    ///
    /// Consumes `self` — the transaction cannot be reused.  On error the
    /// substrate state is unchanged; the caller should start a new transaction
    /// to retry.
    fn commit(self) -> Result<(), SubstrateError>;

    /// Discard the working copy without touching the substrate.
    ///
    /// Consumes `self` — the transaction cannot be reused.
    fn rollback(self) -> Result<(), SubstrateError>;
}
