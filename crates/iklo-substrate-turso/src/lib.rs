#![cfg_attr(not(feature = "turso"), allow(dead_code))]

//! Foundational Turso primitives for the Iklo substrate.
//!
//! This crate provides schema management, a versioned value codec, and
//! retry/ambiguous-commit policy primitives used to build a [`Substrate`]
//! trait implementation in subsequent changes. Everything is gated behind
//! the `turso` feature and is not enabled by default.

#[cfg(feature = "turso")]
use std::fmt;

/// Version-tagged binary codec for values persisted into the bindings BLOB column.
#[cfg(feature = "turso")]
pub mod codec;

/// Idempotent schema bootstrap and schema-version verification.
#[cfg(feature = "turso")]
pub mod schema;

#[cfg(test)]
mod tests;

/// Errors produced by this crate's Turso integration.
///
/// Intentionally minimal today — mirrors the style of
/// [`iklo_substrate::SubstrateError`]. Only present when the `turso` feature
/// is enabled, since it wraps [`turso::Error`].
#[cfg(feature = "turso")]
#[derive(Debug)]
pub enum TursoSubstrateError {
    /// A lower-level Turso database error occurred.
    Turso(turso::Error),
    /// The on-disk schema version stored in `iklo_substrate_meta` does not
    /// match the schema version this crate expects. This is distinct from a
    /// generic [`turso::Error`] so callers (and tests) can distinguish an
    /// incompatible-schema failure from an I/O or driver-level failure.
    SchemaVersionMismatch {
        /// The schema version this build of the crate expects.
        expected: i64,
        /// The schema version found in the database.
        found: i64,
    },
    /// A [`codec`] version tag was not recognized. See the `codec` module's
    /// migration policy docs: with only one version defined today, an
    /// unrecognized tag is always a hard decode error, never a silent
    /// best-effort parse.
    UnsupportedCodecVersion {
        /// The unrecognized version byte found in the payload.
        found: u8,
    },
    /// A [`codec`] payload had a valid version tag but could not otherwise
    /// be decoded (e.g. wrong payload length for the version). Carries a
    /// human-readable description of what went wrong.
    CodecDecodeFailed(String),
}

#[cfg(feature = "turso")]
impl fmt::Display for TursoSubstrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TursoSubstrateError::Turso(err) => write!(f, "turso error: {err}"),
            TursoSubstrateError::SchemaVersionMismatch { expected, found } => write!(
                f,
                "schema version mismatch: expected {expected}, found {found}"
            ),
            TursoSubstrateError::UnsupportedCodecVersion { found } => {
                write!(f, "unsupported codec version tag: {found}")
            }
            TursoSubstrateError::CodecDecodeFailed(reason) => {
                write!(f, "codec decode failed: {reason}")
            }
        }
    }
}

#[cfg(feature = "turso")]
impl std::error::Error for TursoSubstrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TursoSubstrateError::Turso(err) => Some(err),
            TursoSubstrateError::SchemaVersionMismatch { .. } => None,
            TursoSubstrateError::UnsupportedCodecVersion { .. } => None,
            TursoSubstrateError::CodecDecodeFailed(_) => None,
        }
    }
}

#[cfg(feature = "turso")]
impl From<turso::Error> for TursoSubstrateError {
    fn from(err: turso::Error) -> Self {
        TursoSubstrateError::Turso(err)
    }
}

/// Converts a Turso-level error into the backend-agnostic
/// [`iklo_substrate::SubstrateError`] required by the `Transaction` trait
/// contract.
///
/// `SubstrateError` currently has a single variant, `BindingFailed`, so this
/// conversion loses the finer-grained Turso error kind. As `SubstrateError`
/// grows richer variants, this mapping should be revisited.
#[cfg(feature = "turso")]
impl From<TursoSubstrateError> for iklo_substrate::SubstrateError {
    fn from(err: TursoSubstrateError) -> Self {
        iklo_substrate::SubstrateError::BindingFailed(err.to_string())
    }
}

/// Whether a failed operation is safe to retry, or must be surfaced to the
/// caller immediately.
#[cfg(feature = "turso")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    /// The failure is transient/transport-shaped (contention, timeout,
    /// I/O hiccup) — a caller may retry the operation.
    Retryable,
    /// The failure reflects a caller error, a data/schema problem, or an
    /// otherwise non-transient condition — retrying without changing
    /// anything would just fail again. Surface it immediately.
    SurfaceImmediately,
}

/// Classifies a [`TursoSubstrateError`] as [`RetryClass::Retryable`] or
/// [`RetryClass::SurfaceImmediately`].
///
/// ## Mapping table
///
/// This crate's own variants are never transient — they represent a
/// schema/codec mismatch discovered locally, not a database round-trip
/// failure — so they are always `SurfaceImmediately`:
///
/// | `TursoSubstrateError` variant | `RetryClass` | Rationale |
/// |---|---|---|
/// | `SchemaVersionMismatch` | `SurfaceImmediately` | Retrying doesn't change the on-disk schema version. |
/// | `UnsupportedCodecVersion` | `SurfaceImmediately` | The payload's version tag won't change on retry. |
/// | `CodecDecodeFailed` | `SurfaceImmediately` | Malformed bytes stay malformed on retry. |
///
/// For the wrapped `turso::Error` (as of `turso` 0.7.0, whose `Error` enum is
/// NOT `#[non_exhaustive]` — see `turso-0.7.0/src/lib.rs`), each real variant
/// is classified individually rather than lumped into a catch-all, per the
/// following table:
///
/// | `turso::Error` variant | `RetryClass` | Rationale |
/// |---|---|---|
/// | `Busy` | `Retryable` | SQLite/Turso lock contention — the canonical "retry after backoff" case. |
/// | `BusySnapshot` | `Retryable` | MVCC snapshot conflict — a concurrent writer won; retrying against a fresh snapshot can succeed. |
/// | `Interrupt` | `Retryable` | The operation was interrupted (e.g. by a competing statement); nothing about the data is wrong. |
/// | `IoError` | `Retryable` (transient kinds) / `SurfaceImmediately` (others) | I/O failures with transient kinds (`Interrupted`, `WouldBlock`, `TimedOut`, `ConnectionReset`, `ConnectionAborted`) are retryable; permanent kinds (e.g. `PermissionDenied`) are surfaced immediately. |
/// | `Error` | `SurfaceImmediately` | Opaque catch-all string from the underlying engine with no documented transient/permanent semantics. There is no way to distinguish a transient occurrence of this variant from a permanent one without inspecting engine-internal message text, which this crate deliberately avoids depending on. The safe default for an unclassifiable error is to not retry blindly, so this is surfaced immediately; see the note below. |
/// | `Misuse` | `SurfaceImmediately` | Programmer/caller error (e.g. wrong argument count) — retrying without fixing the call site cannot succeed. |
/// | `Constraint` | `SurfaceImmediately` | A SQL constraint (e.g. `UNIQUE`, `CHECK`) was violated — this is a data/logic condition, not a transient one. |
/// | `Readonly` | `SurfaceImmediately` | The connection/database is read-only — retrying the same write will not succeed. |
/// | `DatabaseFull` | `SurfaceImmediately` | Out of disk/storage space — will not resolve itself on the timescale of a retry loop. |
/// | `NotAdb` | `SurfaceImmediately` | The file is not a valid database — a fixed, non-transient condition. |
/// | `Corrupt` | `SurfaceImmediately` | Database corruption — retrying reads/writes against corrupt data is unsafe and won't help. |
/// | `QueryReturnedNoRows` | `SurfaceImmediately` | An absence-of-data condition, not a transport failure; retrying an identical query against unchanged data returns the same (non-)result. |
/// | `ConversionFailure` | `SurfaceImmediately` | A value failed to convert to the requested Rust type — a caller/schema mismatch, not transient. |
/// | `ToSqlConversionFailure` | `SurfaceImmediately` | A value failed to convert *to* SQL — same rationale as `ConversionFailure`. |
///
/// **Documented limitation**: `turso::Error` does *not* split cleanly into
/// "transient vs caller error" for every variant. The catch-all `Error(String)`
/// variant in particular carries no structured error code, so this function
/// makes the conservative choice to classify it as `SurfaceImmediately`
/// rather than inventing a transient-looking special case that the real enum
/// doesn't support. This deliberately trades a few false negatives (some
/// `Error(String)` occurrences may in fact be transient) for the safety of
/// never retrying blindly on an error this crate cannot positively identify
/// as transient.
#[cfg(feature = "turso")]
pub fn classify(err: &TursoSubstrateError) -> RetryClass {
    match err {
        TursoSubstrateError::SchemaVersionMismatch { .. }
        | TursoSubstrateError::UnsupportedCodecVersion { .. }
        | TursoSubstrateError::CodecDecodeFailed(_) => RetryClass::SurfaceImmediately,
        TursoSubstrateError::Turso(inner) => match inner {
            turso::Error::Busy(_) | turso::Error::BusySnapshot(_) => RetryClass::Retryable,
            turso::Error::Interrupt(_) => RetryClass::Retryable,
            turso::Error::IoError(kind, _) => match kind {
                std::io::ErrorKind::Interrupted
                | std::io::ErrorKind::WouldBlock
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted => RetryClass::Retryable,
                _ => RetryClass::SurfaceImmediately,
            },
            _ => RetryClass::SurfaceImmediately,
        },
    }
}

/// Bounded exponential-backoff retry policy.
///
/// Pure/decoupled from any live database: [`RetryPolicy::backoff_for`] is a
/// function of `(attempt, policy)` only — no sleeping, no I/O — so it is
/// trivially unit-testable. Actually sleeping between attempts, and deciding
/// whether an attempt number has exceeded `max_attempts`, is the caller's
/// responsibility (wired in by T016).
#[cfg(feature = "turso")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first, non-retry attempt)
    /// a caller should make before giving up.
    pub max_attempts: u32,
    /// The base backoff duration; attempt 1's backoff is exactly this value.
    pub base_backoff: std::time::Duration,
}

#[cfg(feature = "turso")]
impl RetryPolicy {
    /// Backoff cap: no computed backoff ever exceeds this duration,
    /// regardless of how large `attempt` is. Chosen as a round, human-legible
    /// upper bound suitable for a local/embedded database retry loop (as
    /// opposed to, say, a multi-minute cap appropriate for a remote HTTP
    /// API) — long enough to ride out realistic lock contention, short
    /// enough that a caller isn't left waiting indefinitely.
    pub const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);

    /// Computes the backoff duration for the given 1-indexed attempt number,
    /// using `base_backoff * 2^(attempt - 1)`, capped at [`Self::MAX_BACKOFF`].
    ///
    /// `attempt` is expected to be `>= 1`; `attempt == 0` is treated the same
    /// as `attempt == 1` (no negative exponent, no panic) since there is no
    /// meaningful "0th attempt" backoff.
    ///
    /// The exponent is computed via `saturating_sub`/`checked` arithmetic so
    /// that even a very large `attempt` (e.g. `u32::MAX`) cannot panic or
    /// overflow: `checked_mul`/`checked_pow` failures simply saturate to
    /// [`Self::MAX_BACKOFF`] rather than wrapping or panicking.
    pub fn backoff_for(&self, attempt: u32) -> std::time::Duration {
        let exponent = attempt.saturating_sub(1);
        // 2^exponent as a u32, saturating instead of panicking/overflowing
        // for large exponents (anything >= 32 already saturates to u32::MAX,
        // which will blow past the cap once multiplied anyway).
        let multiplier = 2u32.checked_pow(exponent).unwrap_or(u32::MAX);
        let scaled = self.base_backoff.checked_mul(multiplier);
        match scaled {
            Some(duration) if duration <= Self::MAX_BACKOFF => duration,
            _ => Self::MAX_BACKOFF,
        }
    }
}

/// The outcome of resolving an ambiguous commit result (e.g. after a timeout
/// or dropped connection where the caller cannot tell whether the server
/// applied the operation before the failure occurred).
#[cfg(feature = "turso")]
#[derive(Debug)]
pub enum AmbiguousCommitResolution {
    /// `verify` confirmed the operation was already applied — treat the
    /// original (ambiguous) attempt as a success. Do not retry: retrying
    /// would double-apply.
    AlreadyApplied,
    /// `verify` confirmed the operation did NOT land — it is safe to retry.
    SafeToRetry,
    /// `verify` itself failed, so it is impossible to determine whether the
    /// operation landed. Surface immediately; never guess or retry blindly.
    /// Carries the underlying error for diagnostic propagation.
    VerificationFailed(TursoSubstrateError),
}

/// Resolves an ambiguous commit outcome (FR-013/FR-022: verify the commit
/// outcome before making any retry decision) by running a caller-supplied
/// `verify` step.
///
/// `verify` must return `Ok(true)` if the operation is confirmed to have
/// already landed, `Ok(false)` if it is confirmed to have NOT landed, or
/// `Err(_)` if the outcome could not be determined.
///
/// # Invariant
///
/// This function must never return [`AmbiguousCommitResolution::SafeToRetry`]
/// without `verify` having returned a definitive `Ok(false)`. If `verify`
/// errors, the safe default is [`AmbiguousCommitResolution::VerificationFailed`]
/// (surface immediately), never a blind retry — a networked commit that
/// timed out may have already been applied server-side, and retrying without
/// a confirmed negative would risk double-applying it.
#[cfg(feature = "turso")]
pub fn resolve_ambiguous_commit(
    verify: impl FnOnce() -> Result<bool, TursoSubstrateError>,
) -> AmbiguousCommitResolution {
    match verify() {
        Ok(true) => AmbiguousCommitResolution::AlreadyApplied,
        Ok(false) => AmbiguousCommitResolution::SafeToRetry,
        Err(e) => AmbiguousCommitResolution::VerificationFailed(e),
    }
}
