//! Turso-backed [`Substrate`]/[`Transaction`] implementation.
//!
//! This is the first place the pieces built by the earlier tasks are wired
//! together: the [`schema`](crate::schema) bootstrap/verify, the version-tagged
//! [`Codec`](iklo_substrate::Codec), and the retry/ambiguity policy logic
//! ([`classify`](crate::classify) / [`RetryPolicy`](crate::RetryPolicy) /
//! [`resolve_ambiguous_commit`](crate::resolve_ambiguous_commit)) all live
//! behind this type.
//!
//! # Sync-over-async bridge
//!
//! The [`Substrate`]/[`Transaction`] traits are **synchronous**, but the
//! `turso` client is **async**. [`TursoSubstrate`] owns a current-thread
//! `tokio` runtime and drives every database round-trip through
//! [`Runtime::block_on`](tokio::runtime::Runtime::block_on). A current-thread
//! runtime is sufficient because this crate only ever makes one blocking call
//! at a time — there is no concurrency to schedule — but the runtime's `time`
//! driver is required so the commit retry loop can `tokio::time::sleep` for
//! backoff.
//!
//! # `Result`-less trait methods panic on internal failure (by design)
//!
//! [`Substrate::begin`], [`Substrate::revision`], and [`Substrate::snapshot`]
//! cannot return `Result` — the trait contract in
//! `crates/iklo-substrate/src/lib.rs` gives them infallible signatures. A
//! Turso backend nonetheless performs real I/O and decoding in those methods.
//! When that I/O fails, or a stored value fails to decode, there is no error
//! channel to surface it through, so these methods **panic** with a clear,
//! diagnostic message rather than silently dropping a row, substituting a
//! default, or returning a lie. This is a deliberate contract limitation, not
//! an oversight: an unreadable/corrupt authoritative store is an
//! unrecoverable condition for a method that has promised it cannot fail.
//! [`Transaction::commit`] and [`Transaction::rollback`] *do* return `Result`
//! and therefore never panic on I/O — they surface it.

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;

use iklo_substrate::{Codec, Substrate, SubstrateError, Transaction};
use turso::Value;

use crate::{
    classify, resolve_ambiguous_commit, schema, AmbiguousCommitResolution, RetryClass, RetryPolicy,
    TursoSubstrateError,
};

/// Retry policy used by [`TursoTx::commit`].
///
/// Local-file SQLite-compatible writes rarely contend, so the values are
/// modest: a handful of attempts with a short base backoff. The backoff is
/// only ever exercised for genuinely retryable ([`RetryClass::Retryable`])
/// failures such as lock contention (`Busy`) — see [`classify`].
const COMMIT_RETRY_POLICY: RetryPolicy = RetryPolicy {
    max_attempts: 5,
    base_backoff: Duration::from_millis(10),
};

/// A [`Substrate`] whose authoritative binding state and revision counter are
/// persisted to a local Turso (SQLite-compatible) database file.
///
/// Constructed with [`TursoSubstrate::new`], which bootstraps and verifies the
/// schema before returning. See the [module docs](self) for the sync-over-async
/// bridge and the panic-on-internal-failure contract.
pub struct TursoSubstrate<V> {
    /// Owned runtime that bridges this crate's sync API onto turso's async
    /// client. Current-thread flavor with the `time` driver enabled.
    runtime: tokio::runtime::Runtime,
    /// The open database handle. Kept alive alongside `conn` because the
    /// `turso::Connection` is created from it; dropping the `Database` while a
    /// connection is live is not something the vendored 0.7.0 API documents as
    /// safe, so we hold it for the lifetime of the substrate. The field is
    /// intentionally unread after construction.
    _db: turso::Database,
    /// The connection every query/execute runs through.
    conn: turso::Connection,
    _marker: PhantomData<V>,
}

impl<V> fmt::Debug for TursoSubstrate<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TursoSubstrate").finish_non_exhaustive()
    }
}

impl<V> TursoSubstrate<V> {
    /// Opens (creating if necessary) the local Turso database at `path`,
    /// bootstraps the schema, verifies it, and returns a ready substrate.
    ///
    /// `path` is a **local file path** (or `":memory:"`). This epic is
    /// local-file-only by design (blocker `B001`): there is no remote/sync
    /// mode here. Every `new` call runs [`schema::bootstrap`] followed by
    /// [`schema::verify`], so opening an existing database re-checks its schema
    /// version.
    ///
    /// Returns `Err` — never panics, never silently falls back to an
    /// in-memory/unpersisted mode (FR-007) — if the file cannot be opened, the
    /// tables cannot be created, or the on-disk schema version does not match.
    pub fn new(path: &str) -> Result<Self, TursoSubstrateError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                TursoSubstrateError::Turso(turso::Error::Error(format!(
                    "failed to build tokio runtime for TursoSubstrate: {e}"
                )))
            })?;

        let (db, conn) = runtime.block_on(async {
            let db = turso::Builder::new_local(path).build().await?;
            let conn = db.connect()?;
            schema::bootstrap(&conn).await?;
            schema::verify(&conn).await?;
            Ok::<_, TursoSubstrateError>((db, conn))
        })?;

        Ok(Self {
            runtime,
            _db: db,
            conn,
            _marker: PhantomData,
        })
    }
}

impl<V: Clone + fmt::Debug + Codec> Substrate for TursoSubstrate<V> {
    type Value = V;
    type Tx<'a>
        = TursoTx<'a, V>
    where
        Self: 'a;

    /// Opens a transaction, loading a full working copy of **all** currently
    /// committed bindings — mirroring [`InMemorySubstrate::begin`]'s full
    /// clone of its `HashMap`, not a lazy/partial load.
    ///
    /// Panics if the bindings cannot be read or a stored value fails to decode
    /// — see the [module docs](self) on why a `Result`-less method must panic
    /// on unrecoverable internal failure.
    fn begin(&mut self) -> Self::Tx<'_> {
        let working = self
            .runtime
            .block_on(load_bindings::<V>(&self.conn))
            .unwrap_or_else(|e| panic!("TursoSubstrate::begin failed to load bindings: {e}"));
        TursoTx {
            substrate: self,
            working,
        }
    }

    /// The number of successfully committed transactions, read from
    /// `iklo_substrate_revision`.
    ///
    /// Panics on I/O failure, or if the stored revision is somehow negative
    /// (an internal invariant violation — the counter only ever increments
    /// from 0). See the [module docs](self) on the `Result`-less panic
    /// contract.
    fn revision(&self) -> u64 {
        self.runtime
            .block_on(read_revision(&self.conn))
            .unwrap_or_else(|e| panic!("TursoSubstrate::revision failed to read revision: {e}"))
    }

    /// An owned snapshot of all committed bindings. Same read/decode path as
    /// [`begin`](Self::begin); panics on unrecoverable internal failure.
    fn snapshot(&self) -> HashMap<String, Self::Value> {
        self.runtime
            .block_on(load_bindings::<V>(&self.conn))
            .unwrap_or_else(|e| panic!("TursoSubstrate::snapshot failed to load bindings: {e}"))
    }
}

/// The [`Transaction`] returned by [`TursoSubstrate::begin`]. Holds an
/// in-memory working copy; all database writes happen atomically at
/// [`commit`](Transaction::commit) time.
pub struct TursoTx<'a, V> {
    substrate: &'a mut TursoSubstrate<V>,
    working: HashMap<String, V>,
}

impl<'a, V: fmt::Debug> fmt::Debug for TursoTx<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TursoTx")
            .field("working", &self.working)
            .finish_non_exhaustive()
    }
}

impl<'a, V: Clone + fmt::Debug + Codec> Transaction for TursoTx<'a, V> {
    type Value = V;

    /// Reads from the in-memory working copy — sees this transaction's own
    /// uncommitted writes, no database I/O.
    fn get(&self, name: &str) -> Option<Self::Value> {
        self.working.get(name).cloned()
    }

    /// Writes into the in-memory working copy. The authoritative store is
    /// untouched until [`commit`](Self::commit); no database I/O here.
    fn set(&mut self, name: &str, value: Self::Value) {
        self.working.insert(name.to_owned(), value);
    }

    /// Atomically writes the working copy back to the database and increments
    /// the revision counter.
    ///
    /// The whole write runs inside a real SQL transaction (`BEGIN` … `COMMIT`).
    /// Failures are routed through the crate's retry/ambiguity policy:
    ///
    /// * A failure **before** the `COMMIT` statement is unambiguously *not
    ///   applied*: the transaction is rolled back and the error is
    ///   [`classify`]-ed. `Retryable` errors back off ([`RetryPolicy`]) and
    ///   retry the whole sequence up to `max_attempts`; everything else
    ///   surfaces immediately.
    /// * A failure of the **`COMMIT` statement itself** is *ambiguous* — it may
    ///   have landed before the failure was observed. This is resolved with
    ///   [`resolve_ambiguous_commit`], which re-reads the revision:
    ///   `AlreadyApplied` → success (never retried — this is the double-apply
    ///   guard); `SafeToRetry` → retry; `VerificationFailed` → surface.
    fn commit(self) -> Result<(), SubstrateError> {
        let TursoTx { substrate, working } = self;
        let conn = &substrate.conn;
        let policy = COMMIT_RETRY_POLICY;

        substrate.runtime.block_on(async move {
            let mut attempt: u32 = 1;
            loop {
                // Step 1: snapshot the revision *before* attempting the write,
                // so an ambiguous COMMIT can be resolved by comparing after.
                let revision_before = read_revision(conn).await?;

                // Step 2: run the write inside a real SQL transaction.
                match apply_write(conn, &working).await {
                    Ok(()) => return Ok(()),

                    // Step 3a: failure BEFORE the COMMIT statement — the write
                    // unambiguously did not land. Roll back (ignoring the
                    // rollback's own result) and classify.
                    Err(WriteFailure::BeforeCommit(err)) => {
                        let _ = conn.execute("ROLLBACK", ()).await;
                        match classify(&err) {
                            RetryClass::Retryable if attempt < policy.max_attempts => {
                                tokio::time::sleep(policy.backoff_for(attempt)).await;
                                attempt += 1;
                                continue;
                            }
                            _ => return Err(err.into()),
                        }
                    }

                    // Step 3b: the COMMIT statement itself failed — AMBIGUOUS.
                    // The write may have landed before the error surfaced.
                    // Verify by re-reading the revision, then let
                    // resolve_ambiguous_commit make the (double-apply-safe)
                    // decision. The verification query is awaited here and its
                    // result handed to resolve_ambiguous_commit's synchronous
                    // closure precomputed, since that closure cannot itself
                    // await.
                    Err(WriteFailure::CommitStatement(err)) => {
                        let verification: Result<bool, TursoSubstrateError> =
                            read_revision(conn).await.map(|after| after != revision_before);

                        match resolve_ambiguous_commit(move || verification) {
                            // Confirmed landed: treat as success. Do NOT retry
                            // — retrying would double-apply the write.
                            AmbiguousCommitResolution::AlreadyApplied => return Ok(()),
                            // Confirmed did NOT land: safe to retry. Clear any
                            // dangling transaction first, then back off/retry.
                            AmbiguousCommitResolution::SafeToRetry
                                if attempt < policy.max_attempts =>
                            {
                                let _ = conn.execute("ROLLBACK", ()).await;
                                tokio::time::sleep(policy.backoff_for(attempt)).await;
                                attempt += 1;
                                continue;
                            }
                            // Safe to retry but out of attempts, or verification
                            // failed: surface the original commit error. Never
                            // retry blindly on an unverified ambiguous commit.
                            // A failed COMMIT typically leaves the SQL
                            // transaction open pending an explicit ROLLBACK, so
                            // clear it here (ignoring the rollback's own
                            // result) before returning — otherwise this
                            // long-lived connection would be left with a
                            // dangling open transaction and every subsequent
                            // commit() call would fail at its own BEGIN.
                            AmbiguousCommitResolution::SafeToRetry
                            | AmbiguousCommitResolution::VerificationFailed => {
                                let _ = conn.execute("ROLLBACK", ()).await;
                                return Err(err.into());
                            }
                        }
                    }
                }
            }
        })
    }

    /// Discards the working copy. Issues **zero** database I/O — nothing was
    /// ever written to the database during the transaction (writes only happen
    /// at [`commit`](Self::commit) time), so there is nothing to undo. Matches
    /// `InMemoryTx::rollback`'s no-op semantics exactly.
    fn rollback(self) -> Result<(), SubstrateError> {
        Ok(())
    }
}

/// Distinguishes a write failure that happened *before* the final `COMMIT`
/// (unambiguously not applied) from a failure of the `COMMIT` statement itself
/// (ambiguous — may have landed).
enum WriteFailure {
    /// Any statement before `COMMIT` failed. The transaction did not commit.
    BeforeCommit(TursoSubstrateError),
    /// The `COMMIT` statement failed. Outcome ambiguous.
    CommitStatement(TursoSubstrateError),
}

/// Runs the full write sequence inside a real SQL transaction:
/// `BEGIN`, `DELETE` all bindings, `INSERT` each `(name, encode(value))`,
/// bump the revision, `COMMIT`.
///
/// Correctness over cleverness: bindings are replaced wholesale (delete + row
/// per entry) rather than diffed, and inserts are individual statements rather
/// than a batched multi-row `VALUES` — matching the brief and this epic's
/// "correctness first" performance stance.
async fn apply_write<V: Codec>(
    conn: &turso::Connection,
    working: &HashMap<String, V>,
) -> Result<(), WriteFailure> {
    conn.execute("BEGIN", ())
        .await
        .map_err(|e| WriteFailure::BeforeCommit(e.into()))?;

    conn.execute("DELETE FROM iklo_substrate_bindings", ())
        .await
        .map_err(|e| WriteFailure::BeforeCommit(e.into()))?;

    for (name, value) in working {
        let encoded = value.encode();
        conn.execute(
            "INSERT INTO iklo_substrate_bindings (name, value) VALUES (?1, ?2)",
            (name.clone(), encoded),
        )
        .await
        .map_err(|e| WriteFailure::BeforeCommit(e.into()))?;
    }

    conn.execute(
        "UPDATE iklo_substrate_revision SET revision = revision + 1 WHERE id = 0",
        (),
    )
    .await
    .map_err(|e| WriteFailure::BeforeCommit(e.into()))?;

    conn.execute("COMMIT", ())
        .await
        .map_err(|e| WriteFailure::CommitStatement(e.into()))?;

    Ok(())
}

/// Reads the singleton revision counter from `iklo_substrate_revision`.
/// Shared by [`TursoSubstrate::revision`] and the ambiguous-commit
/// verification step.
async fn read_revision(conn: &turso::Connection) -> Result<u64, TursoSubstrateError> {
    let mut rows = conn
        .query(
            "SELECT revision FROM iklo_substrate_revision WHERE id = 0",
            (),
        )
        .await?;
    let row = rows
        .next()
        .await?
        .ok_or(TursoSubstrateError::Turso(turso::Error::QueryReturnedNoRows))?;

    match row.get_value(0)? {
        Value::Integer(v) => u64::try_from(v).map_err(|_| {
            TursoSubstrateError::CodecDecodeFailed(format!(
                "revision counter is negative ({v}); internal invariant violated"
            ))
        }),
        other => Err(TursoSubstrateError::Turso(turso::Error::Misuse(format!(
            "expected revision to be an integer, found {other:?}"
        )))),
    }
}

/// Loads and decodes the full set of committed bindings from
/// `iklo_substrate_bindings`. Shared by [`TursoSubstrate::begin`] and
/// [`TursoSubstrate::snapshot`].
async fn load_bindings<V: Codec>(
    conn: &turso::Connection,
) -> Result<HashMap<String, V>, TursoSubstrateError> {
    let mut rows = conn
        .query("SELECT name, value FROM iklo_substrate_bindings", ())
        .await?;

    let mut map = HashMap::new();
    while let Some(row) = rows.next().await? {
        let name = match row.get_value(0)? {
            Value::Text(s) => s,
            other => {
                return Err(TursoSubstrateError::Turso(turso::Error::Misuse(format!(
                    "iklo_substrate_bindings.name expected TEXT, found {other:?}"
                ))))
            }
        };
        let blob = match row.get_value(1)? {
            Value::Blob(b) => b,
            other => {
                return Err(TursoSubstrateError::Turso(turso::Error::Misuse(format!(
                    "iklo_substrate_bindings.value for name {name:?} expected BLOB, found {other:?}"
                ))))
            }
        };
        let value = V::decode(&blob)?;
        map.insert(name, value);
    }
    Ok(map)
}
