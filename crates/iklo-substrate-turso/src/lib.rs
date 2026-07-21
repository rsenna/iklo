#![cfg_attr(not(feature = "turso"), allow(dead_code))]

//! Turso-backed substrate implementation for Iklo.
//!
//! This crate provides a `Substrate` trait implementation using Turso as the backing database.
//! The Turso implementation is gated behind the `turso` feature and is not enabled by default.

#[cfg(feature = "turso")]
use std::fmt;

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
}

#[cfg(feature = "turso")]
impl fmt::Display for TursoSubstrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TursoSubstrateError::Turso(err) => write!(f, "turso error: {err}"),
        }
    }
}

#[cfg(feature = "turso")]
impl std::error::Error for TursoSubstrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TursoSubstrateError::Turso(err) => Some(err),
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

#[cfg(all(test, feature = "turso"))]
mod tests {
    use super::*;

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
}
