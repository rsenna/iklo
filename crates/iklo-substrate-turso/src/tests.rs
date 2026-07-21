//! Integration tests for [`crate::schema`] against a real in-memory `turso`
//! database.
//!
//! These tests only make sense with the `turso` feature enabled, since they
//! exercise real `turso::Connection`s. The whole module is a no-op when the
//! feature is off, so `cargo test --workspace` (without `--features turso`)
//! is unaffected.
#![cfg(feature = "turso")]

use turso::Value;

use crate::codec::{Codec, CODEC_VERSION_I64};
use crate::schema;
use crate::TursoSubstrateError;

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
