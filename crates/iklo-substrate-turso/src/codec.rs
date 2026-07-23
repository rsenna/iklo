//! Version-tagged binary codec for values persisted into the
//! `iklo_substrate_bindings.value` `BLOB` column (see [`crate::schema`]).
//!
//! The schema module intentionally knows nothing about the byte layout of
//! stored values; this module owns that layout instead.
//!
//! # Wire format
//!
//! Every encoded value is `[version_tag: u8][payload...]`. The version tag
//! identifies the shape of `payload` so that future value types or format
//! changes are detectable (FR-021) rather than silently misinterpreted.
//!
//! Only one version is defined today: [`CODEC_VERSION_I64`], whose payload is
//! the 8-byte little-endian encoding of an `i64`.
//!
//! # Migration policy (FR-021)
//!
//! With only one version defined, there is no migration path yet: an
//! unrecognized version tag is a hard decode error
//! ([`TursoSubstrateError::UnsupportedCodecVersion`]), never a silent
//! best-effort parse and never a panic. When a second version is introduced,
//! this policy (and this doc comment) should be revisited to describe how
//! older versions get migrated forward.

use crate::TursoSubstrateError;

/// Wire version tag for the `i64` payload shape: version byte followed by
/// the 8-byte little-endian encoding of the value.
pub const CODEC_VERSION_I64: u8 = 1;

/// Converts a value to/from the version-tagged byte format stored in the
/// `iklo_substrate_bindings.value` column.
///
/// Implementations must make `decode` panic-safe for arbitrary,
/// attacker-controlled byte slices (the column is a `BLOB`; a corrupted or
/// foreign write could contain anything) — use safe slice access
/// (`.get()`/`.get(..)`), never indexing that can panic on short input.
pub trait Codec: Sized {
    /// Encodes `self` into the version-tagged wire format.
    fn encode(&self) -> Vec<u8>;

    /// Decodes a value from the version-tagged wire format.
    ///
    /// Returns an error — never panics — if `bytes` is empty, carries an
    /// unrecognized version tag, or has a payload of the wrong length for
    /// the version it declares. See the module-level docs for the migration
    /// policy governing unrecognized versions.
    fn decode(bytes: &[u8]) -> Result<Self, TursoSubstrateError>;
}

impl Codec for i64 {
    fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + 8);
        out.push(CODEC_VERSION_I64);
        out.extend_from_slice(&self.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, TursoSubstrateError> {
        let version = *bytes.first().ok_or_else(|| {
            TursoSubstrateError::CodecDecodeFailed("empty payload: missing version tag".into())
        })?;

        match version {
            CODEC_VERSION_I64 => {
                let payload = bytes.get(1..).unwrap_or(&[]);
                let array: [u8; 8] = payload.try_into().map_err(|_| {
                    TursoSubstrateError::CodecDecodeFailed(format!(
                        "expected 8-byte i64 payload for version {CODEC_VERSION_I64}, got {} bytes",
                        payload.len()
                    ))
                })?;
                Ok(i64::from_le_bytes(array))
            }
            other => Err(TursoSubstrateError::UnsupportedCodecVersion { found: other }),
        }
    }
}
