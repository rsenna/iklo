//! In-memory [`Substrate`] implementation — the reference backend.

use std::collections::HashMap;
use std::fmt;

use crate::{Substrate, SubstrateError, Transaction};

/// A [`Substrate`] backed by a plain `HashMap`. No persistence, no I/O —
/// the reference implementation the contract suite is proven against.
#[derive(Debug, Clone)]
pub struct InMemorySubstrate<V> {
    bindings: HashMap<String, V>,
    revision: u64,
}

impl<V> InMemorySubstrate<V> {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            revision: 0,
        }
    }
}

impl<V> Default for InMemorySubstrate<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone + fmt::Debug> Substrate for InMemorySubstrate<V> {
    type Value = V;
    type Tx<'a> = InMemoryTx<'a, V>
    where
        Self: 'a;

    fn begin(&mut self) -> Self::Tx<'_> {
        let working = self.bindings.clone();
        InMemoryTx {
            substrate: self,
            working,
        }
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn snapshot(&self) -> HashMap<String, Self::Value> {
        self.bindings.clone()
    }
}

/// The [`Transaction`] returned by [`InMemorySubstrate::begin`]. Holds a
/// working copy of the bindings, cloned at `begin` time; writes go to the
/// clone until [`commit`](Transaction::commit) writes it back.
#[derive(Debug)]
pub struct InMemoryTx<'a, V> {
    substrate: &'a mut InMemorySubstrate<V>,
    working: HashMap<String, V>,
}

impl<'a, V: Clone + fmt::Debug> Transaction for InMemoryTx<'a, V> {
    type Value = V;

    fn get(&self, name: &str) -> Option<Self::Value> {
        self.working.get(name).cloned()
    }

    fn set(&mut self, name: &str, value: Self::Value) {
        self.working.insert(name.to_owned(), value);
    }

    fn commit(self) -> Result<(), SubstrateError> {
        self.substrate.bindings = self.working;
        self.substrate.revision += 1;
        Ok(())
    }

    fn rollback(self) -> Result<(), SubstrateError> {
        Ok(())
    }
}
