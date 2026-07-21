//! Cross-crate value codec boundary.
//!
//! [`Codec`] converts a value type to/from the version-tagged byte format that
//! a persistent [`Substrate`](crate::Substrate) backend stores. It lives in
//! `iklo-substrate` — the crate every backend and the runtime already depend
//! on — so that a value type (e.g. `iklo_runtime::Value`) can implement it
//! **without** taking a dependency on any particular backend crate such as
//! `iklo-substrate-turso`.
//!
//! # Fixed error type
//!
//! `decode` returns a fixed, backend-agnostic [`CodecError`] (a plain message)
//! rather than an associated error type. That keeps the trait usable across
//! crate boundaries: an implementer never has to name, or depend on, a
//! backend-specific error enum. A backend that needs its own richer error type
//! can convert via `From<CodecError>` at its `?` sites.
//!
//! # Panic-safety contract
//!
//! `decode` operates on arbitrary, possibly attacker-controlled bytes (a
//! persisted `BLOB` column could contain anything after corruption or a
//! foreign write). Implementations **must** use safe slice access
//! (`.get()`/`.get(..)`), never indexing that can panic on short input, and
//! must return [`CodecError`] — never panic — on empty, truncated, oversized,
//! or otherwise malformed input.

use std::fmt;

/// A backend-agnostic codec decode failure: an empty/truncated/oversized
/// payload, an unrecognized version tag, or any other malformed input.
///
/// Deliberately minimal — a single human-readable message — because it is a
/// cross-crate boundary type. Backends may map it into their own error enum
/// via `From<CodecError>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecError(pub String);

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "codec decode failed: {}", self.0)
    }
}

impl std::error::Error for CodecError {}

/// Converts a value to/from the version-tagged byte format persisted by a
/// [`Substrate`](crate::Substrate) backend.
///
/// See the [module docs](self) for the fixed-error-type rationale and the
/// panic-safety contract that every implementation of [`decode`](Codec::decode)
/// must uphold.
pub trait Codec: Sized {
    /// Encodes `self` into the version-tagged wire format.
    fn encode(&self) -> Vec<u8>;

    /// Decodes a value from the version-tagged wire format.
    ///
    /// Returns [`CodecError`] — never panics — if `bytes` is empty, carries an
    /// unrecognized version tag, or has a payload of the wrong length for the
    /// version it declares.
    fn decode(bytes: &[u8]) -> Result<Self, CodecError>;
}

/// Wire version tag for the `i64` payload shape: a version byte followed by the
/// 8-byte little-endian encoding of the value.
pub const CODEC_VERSION_I64: u8 = 1;

/// Reference [`Codec`] implementation for `i64`.
///
/// This lives here, in the trait's home crate, rather than in a backend crate:
/// Rust's orphan rule forbids `impl Codec for i64` anywhere else, since both
/// `Codec` and `i64` would then be foreign. `i64` is the value type the
/// backend-agnostic contract suite ([`crate::contract`]) is written against, so
/// a primitive reference codec belongs alongside it.
impl Codec for i64 {
    fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + 8);
        out.push(CODEC_VERSION_I64);
        out.extend_from_slice(&self.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        // Panic-safety: only `.first()` / `.get(..)` are used, so an empty,
        // 1-byte, or oversized adversarial slice returns `Err`, never panics.
        let version = *bytes
            .first()
            .ok_or_else(|| CodecError("empty payload: missing version tag".into()))?;

        match version {
            CODEC_VERSION_I64 => {
                let payload = bytes.get(1..).unwrap_or(&[]);
                let array: [u8; 8] = payload.try_into().map_err(|_| {
                    CodecError(format!(
                        "expected 8-byte i64 payload for version {CODEC_VERSION_I64}, got {} bytes",
                        payload.len()
                    ))
                })?;
                Ok(i64::from_le_bytes(array))
            }
            other => Err(CodecError(format!("unsupported codec version tag: {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_codec_round_trips_extremes_and_zero() {
        for value in [i64::MIN, -1, 0, 1, 42, i64::MAX] {
            let encoded = value.encode();
            let decoded = i64::decode(&encoded).expect("round-trip decode should succeed");
            assert_eq!(decoded, value, "round-trip mismatch for {value}");
        }
    }

    #[test]
    fn i64_codec_encodes_with_expected_tag_and_length() {
        let encoded = 7i64.encode();
        assert_eq!(encoded.len(), 1 + 8);
        assert_eq!(encoded[0], CODEC_VERSION_I64);
    }

    #[test]
    fn i64_decode_is_panic_safe_on_adversarial_input() {
        assert!(i64::decode(&[]).is_err(), "empty slice");
        assert!(i64::decode(&[CODEC_VERSION_I64]).is_err(), "tag only");
        assert!(i64::decode(&[CODEC_VERSION_I64, 1, 2, 3]).is_err(), "truncated");
        assert!(
            i64::decode(&[CODEC_VERSION_I64, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_err(),
            "oversized"
        );
        assert!(
            i64::decode(&[0xFF, 0, 0, 0, 0, 0, 0, 0, 0]).is_err(),
            "unknown version tag"
        );
    }
}
