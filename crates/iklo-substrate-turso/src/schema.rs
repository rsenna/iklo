//! Idempotent schema bootstrap and verification for the Turso-backed substrate.
//!
//! This module owns the on-disk table layout used to persist bindings,
//! schema version, and the monotonic revision counter. It intentionally
//! knows nothing about the byte layout of stored values — that is the
//! codec module's responsibility (a later task).

use turso::Value;

use crate::TursoSubstrateError;

/// Current schema version. Bump this whenever the table layout changes in a
/// way that isn't backward compatible, and add a migration path.
pub const SCHEMA_VERSION: i64 = 1;

const CREATE_META_TABLE: &str = "CREATE TABLE IF NOT EXISTS iklo_substrate_meta (\
    id INTEGER PRIMARY KEY CHECK (id = 0), \
    schema_version INTEGER NOT NULL\
)";

const CREATE_BINDINGS_TABLE: &str = "CREATE TABLE IF NOT EXISTS iklo_substrate_bindings (\
    name TEXT PRIMARY KEY, \
    value BLOB NOT NULL\
)";

const CREATE_REVISION_TABLE: &str = "CREATE TABLE IF NOT EXISTS iklo_substrate_revision (\
    id INTEGER PRIMARY KEY CHECK (id = 0), \
    revision INTEGER NOT NULL\
)";

const INSERT_META_ROW: &str =
    "INSERT OR IGNORE INTO iklo_substrate_meta (id, schema_version) VALUES (0, ?1)";

const INSERT_REVISION_ROW: &str =
    "INSERT OR IGNORE INTO iklo_substrate_revision (id, revision) VALUES (0, 0)";

const SELECT_SCHEMA_VERSION: &str = "SELECT schema_version FROM iklo_substrate_meta WHERE id = 0";

/// Idempotently creates the substrate's on-disk schema.
///
/// Creates the `iklo_substrate_meta`, `iklo_substrate_bindings`, and
/// `iklo_substrate_revision` tables via `CREATE TABLE IF NOT EXISTS`, and, if
/// the singleton meta/revision rows don't already exist, inserts them
/// (`schema_version = SCHEMA_VERSION`, `revision = 0`) via `INSERT OR
/// IGNORE`. Calling this function repeatedly against the same database is a
/// no-op after the first call: no error, no data change.
pub async fn bootstrap(conn: &turso::Connection) -> Result<(), TursoSubstrateError> {
    conn.execute(CREATE_META_TABLE, ()).await?;
    conn.execute(CREATE_BINDINGS_TABLE, ()).await?;
    conn.execute(CREATE_REVISION_TABLE, ()).await?;

    conn.execute(INSERT_META_ROW, [SCHEMA_VERSION]).await?;
    conn.execute(INSERT_REVISION_ROW, ()).await?;

    Ok(())
}

/// Verifies that the on-disk schema version matches [`SCHEMA_VERSION`].
///
/// Returns `Ok(())` if the stored `schema_version` in `iklo_substrate_meta`
/// matches. Returns
/// [`TursoSubstrateError::SchemaVersionMismatch`] if the database was
/// bootstrapped with a different (older or newer) schema version than this
/// build of the crate expects.
pub async fn verify(conn: &turso::Connection) -> Result<(), TursoSubstrateError> {
    let mut rows = conn.query(SELECT_SCHEMA_VERSION, ()).await?;

    let row = rows
        .next()
        .await?
        .ok_or_else(|| TursoSubstrateError::Turso(turso::Error::QueryReturnedNoRows))?;

    let found = match row.get_value(0)? {
        Value::Integer(version) => version,
        other => {
            return Err(TursoSubstrateError::Turso(turso::Error::Misuse(format!(
                "expected schema_version to be an integer, found {other:?}"
            ))));
        }
    };

    if found == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(TursoSubstrateError::SchemaVersionMismatch {
            expected: SCHEMA_VERSION,
            found,
        })
    }
}
