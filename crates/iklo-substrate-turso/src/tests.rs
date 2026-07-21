//! Integration tests for [`crate::schema`] against a real in-memory `turso`
//! database.
//!
//! These tests only make sense with the `turso` feature enabled, since they
//! exercise real `turso::Connection`s. The whole module is a no-op when the
//! feature is off, so `cargo test --workspace` (without `--features turso`)
//! is unaffected.
#![cfg(feature = "turso")]

use turso::Value;

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
