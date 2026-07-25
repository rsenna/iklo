//! Integration tests for [`crate::schema`] against a real in-memory `turso`
//! database.
//!
//! These tests only make sense with the `turso` feature enabled, since they
//! exercise real `turso::Connection`s. The whole module is a no-op when the
//! feature is off, so `cargo test --workspace` (without `--features turso`)
//! is unaffected.
#![cfg(feature = "turso")]

use std::time::Duration;

use turso::Value;

use iklo_substrate::{Substrate, Transaction};

use crate::codec::{Codec, CODEC_VERSION_I64};
use crate::schema;
use crate::substrate::TursoSubstrate;
use crate::{
    classify, resolve_ambiguous_commit, AmbiguousCommitResolution, RetryClass, RetryPolicy,
    TursoSubstrateError,
};

/// Returns a unique, not-yet-existing temp file path for a persistent Turso
/// database. No `tempfile` crate is in the workspace, so we build a unique
/// name under `std::env::temp_dir()` ourselves; `TempDbPath`'s `Drop` removes
/// the file (and Turso's WAL sidecars) on scope exit.
fn unique_db_path(tag: &str) -> TempDbPath {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut path = std::env::temp_dir();
    path.push(format!(
        "iklo-turso-test-{tag}-{}-{nanos}-{seq}.db",
        std::process::id()
    ));
    TempDbPath(path)
}

/// Owns a temp database path and cleans it (plus Turso's `-wal`/`-shm`
/// sidecars) up on drop, so tests don't litter the temp dir.
struct TempDbPath(std::path::PathBuf);

impl TempDbPath {
    fn as_str(&self) -> &str {
        self.0.to_str().expect("temp path is valid UTF-8")
    }
}

impl Drop for TempDbPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        for suffix in ["-wal", "-shm"] {
            let mut sidecar = self.0.clone();
            sidecar.as_mut_os_string().push(suffix);
            let _ = std::fs::remove_file(&sidecar);
        }
    }
}

#[test]
fn turso_error_converts_to_turso_substrate_error() {
    let turso_err = turso::Error::Misuse("boom".to_string());
    let wrapped: TursoSubstrateError = turso_err.into();
    assert_eq!(wrapped.to_string(), "turso error: boom");
}

#[test]
fn turso_substrate_error_converts_to_substrate_error() {
    let wrapped = TursoSubstrateError::Turso(turso::Error::Misuse("boom".to_string()));
    let substrate_err: iklo_substrate::SubstrateError = wrapped.into();
    assert_eq!(
        substrate_err,
        iklo_substrate::SubstrateError::BindingFailed("turso error: boom".to_string())
    );
}

/// Opens a fresh, isolated in-memory database. Each call gets its own
/// database, so tests never share state.
async fn fresh_connection() -> turso::Connection {
    let db = turso::Builder::new_local(":memory:")
        .build()
        .await
        .expect("failed to build in-memory turso database");
    db.connect().expect("failed to open connection")
}

/// Reads the singleton `schema_version` from `iklo_substrate_meta`.
async fn read_schema_version(conn: &turso::Connection) -> i64 {
    let mut rows = conn
        .query(
            "SELECT schema_version FROM iklo_substrate_meta WHERE id = 0",
            (),
        )
        .await
        .expect("query schema_version failed");
    let row = rows
        .next()
        .await
        .expect("fetching schema_version row failed")
        .expect("expected exactly one iklo_substrate_meta row");
    match row
        .get_value(0)
        .expect("failed to read schema_version column")
    {
        Value::Integer(v) => v,
        other => panic!("expected schema_version to be an integer, found {other:?}"),
    }
}

/// Reads the singleton `revision` from `iklo_substrate_revision`.
async fn read_revision(conn: &turso::Connection) -> i64 {
    let mut rows = conn
        .query(
            "SELECT revision FROM iklo_substrate_revision WHERE id = 0",
            (),
        )
        .await
        .expect("query revision failed");
    let row = rows
        .next()
        .await
        .expect("fetching revision row failed")
        .expect("expected exactly one iklo_substrate_revision row");
    match row.get_value(0).expect("failed to read revision column") {
        Value::Integer(v) => v,
        other => panic!("expected revision to be an integer, found {other:?}"),
    }
}

/// Counts rows in a table with no arguments (used to check the singleton
/// meta/revision tables have exactly one row, not duplicated).
async fn row_count(conn: &turso::Connection, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let mut rows = conn.query(&sql, ()).await.expect("row count query failed");
    let row = rows
        .next()
        .await
        .expect("fetching row count failed")
        .expect("COUNT(*) always returns a row");
    match row.get_value(0).expect("failed to read count column") {
        Value::Integer(v) => v,
        other => panic!("expected COUNT(*) to be an integer, found {other:?}"),
    }
}

#[tokio::test]
async fn bootstrap_is_idempotent() {
    let conn = fresh_connection().await;

    schema::bootstrap(&conn)
        .await
        .expect("first bootstrap call should succeed");

    assert_eq!(row_count(&conn, "iklo_substrate_meta").await, 1);
    assert_eq!(row_count(&conn, "iklo_substrate_revision").await, 1);
    assert_eq!(read_schema_version(&conn).await, schema::SCHEMA_VERSION);
    assert_eq!(read_revision(&conn).await, 0);

    schema::bootstrap(&conn)
        .await
        .expect("second bootstrap call should also succeed (no-op)");

    // The second call must not have changed anything: still exactly one row
    // in each singleton table, and the same schema_version/revision values.
    assert_eq!(
        row_count(&conn, "iklo_substrate_meta").await,
        1,
        "bootstrap must not duplicate the iklo_substrate_meta row"
    );
    assert_eq!(
        row_count(&conn, "iklo_substrate_revision").await,
        1,
        "bootstrap must not duplicate the iklo_substrate_revision row"
    );
    assert_eq!(read_schema_version(&conn).await, schema::SCHEMA_VERSION);
    assert_eq!(read_revision(&conn).await, 0);
}

#[tokio::test]
async fn verify_before_bootstrap_returns_clear_error_without_panicking() {
    let conn = fresh_connection().await;

    // No bootstrap() call: iklo_substrate_meta does not exist yet.
    let err = schema::verify(&conn)
        .await
        .expect_err("verify() against a never-bootstrapped database must return an error");

    // Must be distinguishable from a schema-version mismatch (this is a
    // "not bootstrapped at all" failure, not an incompatible-version one),
    // and its Display must not panic and must mention the underlying cause.
    match &err {
        TursoSubstrateError::Turso(inner) => {
            let message = inner.to_string();
            assert!(
                message.to_lowercase().contains("no such table"),
                "expected a 'no such table' error from the missing iklo_substrate_meta table, got: {message}"
            );
        }
        other => panic!(
            "expected TursoSubstrateError::Turso for a missing table, got {other:?}"
        ),
    }

    // Display must render without panicking and should be a legible message.
    let rendered = err.to_string();
    assert!(rendered.starts_with("turso error: "));
}

#[tokio::test]
async fn verify_succeeds_immediately_after_bootstrap_on_fresh_database() {
    let conn = fresh_connection().await;

    schema::bootstrap(&conn)
        .await
        .expect("bootstrap should succeed on a fresh database");

    schema::verify(&conn)
        .await
        .expect("verify must return Ok(()) immediately after a fresh bootstrap");

    // Confirm the state verify() checked against is what we expect: exactly
    // one meta row, at the current schema version.
    assert_eq!(row_count(&conn, "iklo_substrate_meta").await, 1);
    assert_eq!(read_schema_version(&conn).await, schema::SCHEMA_VERSION);
}

#[tokio::test]
async fn verify_rejects_incompatible_schema_version() {
    let conn = fresh_connection().await;

    schema::bootstrap(&conn)
        .await
        .expect("bootstrap should succeed on a fresh database");

    // A fresh bootstrap should verify cleanly.
    schema::verify(&conn)
        .await
        .expect("verify should succeed immediately after bootstrap");

    // Simulate an old/future schema by directly rewriting the stored version.
    let stale_version = schema::SCHEMA_VERSION + 1;
    conn.execute(
        "UPDATE iklo_substrate_meta SET schema_version = ?1 WHERE id = 0",
        [stale_version],
    )
    .await
    .expect("directly updating schema_version should succeed");

    // Sanity-check the simulated mismatch actually landed before asserting
    // on verify()'s behavior.
    assert_eq!(read_schema_version(&conn).await, stale_version);

    let err = schema::verify(&conn)
        .await
        .expect_err("verify must reject a mismatched schema_version");

    match err {
        TursoSubstrateError::SchemaVersionMismatch { expected, found } => {
            assert_eq!(expected, schema::SCHEMA_VERSION);
            assert_eq!(found, stale_version);
        }
        other => panic!("expected SchemaVersionMismatch, got {other:?}"),
    }
}

// --- codec (T007/T008) ---
//
// `Codec` operates on plain `&[u8]`/`Vec<u8>` with no I/O, so these tests
// don't need a live `turso` database. They're kept in this file (which is
// already `#[cfg(feature = "turso")]`-gated at the module level) rather than
// split into a separate always-on test module, since the brief says either
// placement is fine and this keeps all of this crate's tests in one place.

#[test]
fn i64_codec_round_trips_negative_zero_and_positive_values() {
    for value in [i64::MIN, -1, 0, 1, 42, i64::MAX] {
        let encoded = value.encode();
        let decoded = i64::decode(&encoded).expect("round-trip decode should succeed");
        assert_eq!(decoded, value, "round-trip mismatch for {value}");
    }
}

#[test]
fn i64_codec_encodes_with_expected_version_tag_and_length() {
    let encoded = 7i64.encode();
    assert_eq!(encoded.len(), 1 + 8, "expected 1-byte tag + 8-byte payload");
    assert_eq!(encoded[0], CODEC_VERSION_I64);
}

#[test]
fn i64_decode_rejects_unknown_version_tag() {
    // Valid-length payload, but a version byte that doesn't exist.
    let bytes = [0xFFu8, 0, 0, 0, 0, 0, 0, 0, 0];

    let err = i64::decode(&bytes).expect_err("unknown version tag must be rejected");

    match err {
        TursoSubstrateError::UnsupportedCodecVersion { found } => assert_eq!(found, 0xFF),
        other => panic!("expected UnsupportedCodecVersion, got {other:?}"),
    }
}

#[test]
fn i64_decode_rejects_truncated_payload() {
    // Valid version tag, but only 3 payload bytes instead of 8.
    let bytes = [CODEC_VERSION_I64, 1, 2, 3];

    let err = i64::decode(&bytes).expect_err("truncated payload must be rejected");

    match err {
        TursoSubstrateError::CodecDecodeFailed(_) => {}
        other => panic!("expected CodecDecodeFailed, got {other:?}"),
    }
}

#[test]
fn i64_decode_rejects_oversized_payload() {
    // Valid version tag, but too many payload bytes.
    let bytes = [CODEC_VERSION_I64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let err = i64::decode(&bytes).expect_err("oversized payload must be rejected");

    match err {
        TursoSubstrateError::CodecDecodeFailed(_) => {}
        other => panic!("expected CodecDecodeFailed, got {other:?}"),
    }
}

#[test]
fn i64_decode_does_not_panic_on_empty_slice() {
    let err = i64::decode(&[]).expect_err("empty slice must be rejected, not panic");
    match err {
        TursoSubstrateError::CodecDecodeFailed(_) => {}
        other => panic!("expected CodecDecodeFailed, got {other:?}"),
    }
}

#[test]
fn i64_decode_does_not_panic_on_single_byte_slice() {
    // Only the version tag, no payload at all.
    let err = i64::decode(&[CODEC_VERSION_I64])
        .expect_err("version-tag-only slice must be rejected, not panic");
    match err {
        TursoSubstrateError::CodecDecodeFailed(_) => {}
        other => panic!("expected CodecDecodeFailed, got {other:?}"),
    }
}

// --- retry classification, backoff policy, ambiguous-commit resolution (T009/T010) ---
//
// Pure policy logic, no live database required, but kept in this
// feature-gated file alongside the rest of the crate's tests per the brief.

#[test]
fn classify_treats_busy_as_retryable() {
    // `Busy` is the canonical SQLite/Turso lock-contention error — a real,
    // constructible `turso::Error` variant (not fabricated).
    let err = TursoSubstrateError::Turso(turso::Error::Busy("database is locked".to_string()));
    assert_eq!(classify(&err), RetryClass::Retryable);
}

#[test]
fn classify_treats_busy_snapshot_and_interrupt_and_io_error_as_retryable() {
    let busy_snapshot =
        TursoSubstrateError::Turso(turso::Error::BusySnapshot("snapshot conflict".to_string()));
    assert_eq!(classify(&busy_snapshot), RetryClass::Retryable);

    let interrupt =
        TursoSubstrateError::Turso(turso::Error::Interrupt("interrupted".to_string()));
    assert_eq!(classify(&interrupt), RetryClass::Retryable);

    let io_error = TursoSubstrateError::Turso(turso::Error::IoError(
        std::io::ErrorKind::TimedOut,
        "read",
    ));
    assert_eq!(classify(&io_error), RetryClass::Retryable);
}

#[test]
fn classify_treats_permanent_io_error_as_surface_immediately() {
    let err = TursoSubstrateError::Turso(turso::Error::IoError(
        std::io::ErrorKind::PermissionDenied,
        "open",
    ));
    assert_eq!(classify(&err), RetryClass::SurfaceImmediately);
}

#[test]
fn classify_treats_misuse_as_surface_immediately() {
    // `Misuse` is a caller/programmer error — retrying without fixing the
    // call site cannot succeed.
    let err = TursoSubstrateError::Turso(turso::Error::Misuse("bad argument count".to_string()));
    assert_eq!(classify(&err), RetryClass::SurfaceImmediately);
}

#[test]
fn classify_treats_constraint_readonly_corrupt_and_opaque_error_as_surface_immediately() {
    let constraint =
        TursoSubstrateError::Turso(turso::Error::Constraint("UNIQUE violated".to_string()));
    assert_eq!(classify(&constraint), RetryClass::SurfaceImmediately);

    let readonly = TursoSubstrateError::Turso(turso::Error::Readonly("read-only db".to_string()));
    assert_eq!(classify(&readonly), RetryClass::SurfaceImmediately);

    let corrupt = TursoSubstrateError::Turso(turso::Error::Corrupt("corrupt page".to_string()));
    assert_eq!(classify(&corrupt), RetryClass::SurfaceImmediately);

    // The opaque catch-all `Error(String)` variant carries no structured
    // error code, so the documented, conservative default is
    // SurfaceImmediately rather than a blind retry.
    let opaque = TursoSubstrateError::Turso(turso::Error::Error("unspecified".to_string()));
    assert_eq!(classify(&opaque), RetryClass::SurfaceImmediately);
}

#[test]
fn classify_treats_this_crates_own_error_variants_as_surface_immediately() {
    let schema_mismatch = TursoSubstrateError::SchemaVersionMismatch {
        expected: 1,
        found: 2,
    };
    assert_eq!(classify(&schema_mismatch), RetryClass::SurfaceImmediately);

    let unsupported_codec = TursoSubstrateError::UnsupportedCodecVersion { found: 0xFF };
    assert_eq!(classify(&unsupported_codec), RetryClass::SurfaceImmediately);

    let codec_decode_failed = TursoSubstrateError::CodecDecodeFailed("bad payload".to_string());
    assert_eq!(
        classify(&codec_decode_failed),
        RetryClass::SurfaceImmediately
    );
}

#[test]
fn classify_uses_a_real_turso_error_obtained_from_a_live_database() {
    // Like T005/T006, obtain a real `turso::Error` by triggering one against
    // an in-memory database rather than hand-constructing every case.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let err = rt.block_on(async {
        let conn = fresh_connection().await;
        // No such table exists on a never-bootstrapped, freshly-created
        // in-memory database.
        match conn.query("SELECT * FROM does_not_exist", ()).await {
            Ok(_) => panic!("querying a nonexistent table must fail"),
            Err(err) => err,
        }
    });

    let wrapped: TursoSubstrateError = err.into();
    // A "no such table" failure is a caller/schema-mismatch condition, not a
    // transient one — retrying an identical query against an unchanged
    // schema will fail identically every time.
    assert_eq!(classify(&wrapped), RetryClass::SurfaceImmediately);
}

#[test]
fn retry_policy_backoff_follows_exponential_formula_before_the_cap() {
    let policy = RetryPolicy {
        max_attempts: 10,
        base_backoff: Duration::from_millis(100),
    };

    assert_eq!(policy.backoff_for(1), Duration::from_millis(100));
    assert_eq!(policy.backoff_for(2), Duration::from_millis(200));
    assert_eq!(policy.backoff_for(3), Duration::from_millis(400));
    assert_eq!(policy.backoff_for(4), Duration::from_millis(800));
}

#[test]
fn retry_policy_backoff_treats_attempt_zero_like_attempt_one() {
    let policy = RetryPolicy {
        max_attempts: 10,
        base_backoff: Duration::from_millis(100),
    };

    assert_eq!(policy.backoff_for(0), policy.backoff_for(1));
}

#[test]
fn retry_policy_backoff_is_capped_and_does_not_grow_unbounded() {
    let policy = RetryPolicy {
        max_attempts: 10,
        base_backoff: Duration::from_millis(100),
    };

    // Well past the point where 100ms * 2^(attempt-1) would exceed the cap.
    assert_eq!(policy.backoff_for(20), RetryPolicy::MAX_BACKOFF);

    // A very high attempt number must not panic (overflow) and must still
    // be clamped to the cap, not grow unbounded.
    assert_eq!(policy.backoff_for(u32::MAX), RetryPolicy::MAX_BACKOFF);
}

#[test]
fn resolve_ambiguous_commit_returns_already_applied_when_verify_confirms_it_landed() {
    let resolution = resolve_ambiguous_commit(|| Ok(true));
    assert!(matches!(
        resolution,
        AmbiguousCommitResolution::AlreadyApplied
    ));
}

#[test]
fn resolve_ambiguous_commit_returns_safe_to_retry_when_verify_confirms_it_did_not_land() {
    let resolution = resolve_ambiguous_commit(|| Ok(false));
    assert!(matches!(
        resolution,
        AmbiguousCommitResolution::SafeToRetry
    ));
}

#[test]
fn resolve_ambiguous_commit_returns_verification_failed_when_verify_errors_and_never_retries() {
    // The core invariant: if `verify` itself cannot determine the outcome,
    // the function must surface immediately, NOT signal a retry — a
    // networked commit that timed out may have already landed server-side,
    // and retrying without a confirmed negative risks double-applying it.
    let verify_err = TursoSubstrateError::Turso(turso::Error::IoError(
        std::io::ErrorKind::ConnectionReset,
        "verify",
    ));
    let resolution = resolve_ambiguous_commit(|| Err(verify_err));
    assert!(matches!(
        resolution,
        AmbiguousCommitResolution::VerificationFailed(_)
    ));
    assert!(!matches!(
        resolution,
        AmbiguousCommitResolution::SafeToRetry
    ));
}

// --- TursoSubstrate end-to-end (T011/T012) ---
//
// These exercise the real `TursoSubstrate<i64>` against a real local database
// file, driving its own owned tokio runtime — so they are plain `#[test]`s,
// not `#[tokio::test]`s (the substrate owns the runtime; a test-level runtime
// would nest).

/// T011 (FR-004, spec User Story 1): a committed binding survives dropping the
/// entire substrate instance and re-opening a brand-new one against the same
/// file. This is the core persistence-across-restart guarantee.
#[test]
fn commit_persists_across_a_fresh_instance() {
    let path = unique_db_path("persist");

    {
        let mut substrate =
            TursoSubstrate::<i64>::new(path.as_str()).expect("opening a fresh database must work");
        let mut tx = substrate.begin();
        tx.set("x", 42);
        tx.commit().expect("commit of a simple binding must succeed");
        // Substrate (and its connection/runtime) dropped here at end of scope.
    }

    // A genuinely new instance: new runtime, new Database, new Connection,
    // same file on disk. Nothing is reused from the first instance.
    let reopened =
        TursoSubstrate::<i64>::new(path.as_str()).expect("re-opening the same file must work");
    assert_eq!(
        reopened.snapshot().get("x"),
        Some(&42),
        "a committed binding must persist across a full drop + fresh re-open"
    );
    assert_eq!(
        reopened.revision(),
        1,
        "the revision counter must also persist across re-open"
    );
}

/// T011: a rolled-back transaction leaves no trace — a subsequent transaction
/// on the same (still-open) substrate does not see the discarded write.
#[test]
fn rollback_is_invisible_to_later_transactions() {
    let path = unique_db_path("rollback");
    let mut substrate =
        TursoSubstrate::<i64>::new(path.as_str()).expect("opening a fresh database must work");

    let mut tx = substrate.begin();
    tx.set("x", 42);
    tx.rollback().expect("rollback must succeed");

    let tx = substrate.begin();
    assert_eq!(
        tx.get("x"),
        None,
        "a rolled-back write must not be visible to a later transaction"
    );
    drop(tx);

    assert_eq!(
        substrate.revision(),
        0,
        "rollback must not increment the revision counter"
    );
    assert!(
        substrate.snapshot().is_empty(),
        "a rolled-back write must not reach the authoritative store"
    );
}

/// T011: revision starts at 0 on a fresh database, increments by exactly 1 per
/// successful commit (covering two commits to confirm monotonic increment, not
/// just 0->1), and does not move on rollback.
#[test]
fn revision_is_monotonic_across_commits_and_ignores_rollback() {
    let path = unique_db_path("revision");
    let mut substrate =
        TursoSubstrate::<i64>::new(path.as_str()).expect("opening a fresh database must work");

    assert_eq!(substrate.revision(), 0, "fresh database starts at revision 0");

    let mut tx = substrate.begin();
    tx.set("a", 1);
    tx.commit().expect("first commit must succeed");
    assert_eq!(substrate.revision(), 1, "first commit -> revision 1");

    let mut tx = substrate.begin();
    tx.set("b", 2);
    tx.commit().expect("second commit must succeed");
    assert_eq!(
        substrate.revision(),
        2,
        "second commit -> revision 2 (monotonic, +1 each)"
    );

    let mut tx = substrate.begin();
    tx.set("c", 3);
    tx.rollback().expect("rollback must succeed");
    assert_eq!(
        substrate.revision(),
        2,
        "rollback must not change the revision counter"
    );
}

/// T012 (reinterpreted for local-file mode per blocker B001): a path that
/// cannot be opened surfaces an explicit `Err` — not a panic, not a silent
/// fallback to an in-memory/unpersisted substrate (FR-007).
#[test]
fn new_with_unusable_path_surfaces_an_error() {
    // A path under a directory that does not exist and that `new` has no
    // business creating. Opening/bootstrapping must fail with an I/O error.
    let bogus =
        "/iklo-nonexistent-root-dir-xyz/definitely/not/here/substrate.db";

    let result = TursoSubstrate::<i64>::new(bogus);
    assert!(
        result.is_err(),
        "an unusable database path must return Err, not Ok"
    );
}
