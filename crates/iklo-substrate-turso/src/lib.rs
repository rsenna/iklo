#![cfg_attr(not(feature = "turso"), allow(dead_code))]

//! Turso-backed substrate implementation for Iklo.
//!
//! This crate provides a `Substrate` trait implementation using Turso as the backing database.
//! The Turso implementation is gated behind the `turso` feature and is not enabled by default.

#[cfg(feature = "turso")]
use std::fmt;

#[cfg(feature = "turso")]
pub mod codec;

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
