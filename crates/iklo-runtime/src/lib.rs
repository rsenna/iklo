// HashMap used only in the public bindings() return type — internal storage lives behind the Substrate trait.
use std::collections::HashMap;

use iklo_ast::{BinOp, Expr, Program, Spanned};
use iklo_substrate::{Codec, CodecError, Substrate, SubstrateError, Transaction};

/// The default in-memory substrate backing [`RuntimeImage::new`]. Public so
/// callers (e.g. the CLI) can name `RuntimeImage`'s default backend explicitly.
pub type IkloSubstrate = iklo_substrate::memory::InMemorySubstrate<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
}

/// Wire version tag for [`Value::Number`]: a version byte followed by the
/// 8-byte little-endian encoding of the `f64`. This is a small, self-contained
/// format owned by `iklo-runtime`, deliberately independent of the `i64`
/// codec in `iklo-substrate-turso` (each value type owns its own layout).
const CODEC_VERSION_NUMBER: u8 = 1;

impl Codec for Value {
    fn encode(&self) -> Vec<u8> {
        match self {
            Value::Number(n) => {
                let mut out = Vec::with_capacity(1 + 8);
                out.push(CODEC_VERSION_NUMBER);
                out.extend_from_slice(&n.to_le_bytes());
                out
            }
        }
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        // Panic-safety (identical bar to the `i64` codec): only `.first()` /
        // `.get(..)` are used, so an empty, 1-byte, or oversized adversarial
        // slice returns `Err` — never panics.
        let version = *bytes
            .first()
            .ok_or_else(|| CodecError("empty payload: missing version tag".into()))?;

        match version {
            CODEC_VERSION_NUMBER => {
                let payload = bytes.get(1..).unwrap_or(&[]);
                let array: [u8; 8] = payload.try_into().map_err(|_| {
                    CodecError(format!(
                        "expected 8-byte f64 payload for Value::Number version \
                         {CODEC_VERSION_NUMBER}, got {} bytes",
                        payload.len()
                    ))
                })?;
                Ok(Value::Number(f64::from_le_bytes(array)))
            }
            other => Err(CodecError(format!(
                "unrecognized Value codec version tag: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

impl From<SubstrateError> for RuntimeError {
    fn from(err: SubstrateError) -> Self {
        RuntimeError::new(err.to_string())
    }
}

/// The live interpreter image. Generic over the [`Substrate`] backend, with a
/// default type parameter of [`IkloSubstrate`] (the in-memory backend) so that
/// existing callers writing `RuntimeImage` / `RuntimeImage::new()` keep
/// compiling and behaving exactly as before. Inject any other backend (e.g. a
/// Turso-backed one) with [`with_substrate`](RuntimeImage::with_substrate).
#[derive(Debug, Clone)]
pub struct RuntimeImage<S: Substrate<Value = Value> = IkloSubstrate> {
    substrate: S,
}

impl Default for RuntimeImage<IkloSubstrate> {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeImage<IkloSubstrate> {
    /// Creates a runtime image backed by a fresh in-memory substrate.
    /// Behaviourally identical to before this type became generic (FR-002).
    pub fn new() -> Self {
        Self {
            substrate: IkloSubstrate::new(),
        }
    }
}

impl<S: Substrate<Value = Value>> RuntimeImage<S> {
    /// Creates a runtime image backed by an arbitrary, caller-supplied
    /// [`Substrate`] (e.g. a Turso-backed one).
    pub fn with_substrate(substrate: S) -> Self {
        Self { substrate }
    }

    pub fn revision(&self) -> u64 {
        self.substrate.revision()
    }

    pub fn bindings(&self) -> HashMap<String, Value> {
        self.substrate.snapshot()
    }

    pub fn eval_in_tx(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut tx = self.substrate.begin();
        match eval_program(&mut tx, program) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(err) => {
                // Rollback failure is secondary to the evaluation error that
                // triggered it; InMemorySubstrate's rollback never fails, but
                // don't let a future backend's rollback error mask `err`.
                let _ = tx.rollback();
                Err(err)
            }
        }
    }
}

fn eval_program(
    tx: &mut impl Transaction<Value = Value>,
    program: &Program,
) -> Result<Value, RuntimeError> {
    let mut last = Value::Number(0.0);
    for expr in program {
        last = eval_expr(tx, expr)?;
    }
    Ok(last)
}

fn eval_expr(
    tx: &mut impl Transaction<Value = Value>,
    expr: &Spanned<Expr>,
) -> Result<Value, RuntimeError> {
    match &expr.node {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::LexRef(name) => tx
            .get(name)
            .ok_or_else(|| RuntimeError::new(format!("undefined :{name}"))),
        Expr::Let { name, value } => {
            let v = eval_expr(tx, value)?;
            tx.set(name, v.clone());
            Ok(v)
        }
        Expr::Binary { op, left, right } => {
            let left = eval_expr(tx, left)?;
            let right = eval_expr(tx, right)?;
            eval_binop(*op, left, right)
        }
    }
}

fn eval_binop(op: BinOp, left: Value, right: Value) -> Result<Value, RuntimeError> {
    let (l, r) = match (left, right) {
        (Value::Number(l), Value::Number(r)) => (l, r),
    };

    let value = match op {
        BinOp::Add => l + r,
        BinOp::Sub => l - r,
        BinOp::Mul => l * r,
        BinOp::Div => {
            if r == 0.0 {
                return Err(RuntimeError::new("division by zero"));
            }
            l / r
        }
    };
    Ok(Value::Number(value))
}

#[cfg(test)]
mod tests {
    use iklo_parser::parse;

    use super::*;

    #[test]
    fn rollback_keeps_image_unchanged() {
        let mut image = RuntimeImage::new();
        let setup = parse("let :x be 10").expect("parse");
        image.eval_in_tx(&setup).expect("eval");

        let bad = parse("let :y be :x / 0").expect("parse");
        let err = image.eval_in_tx(&bad).expect_err("must fail");
        assert!(err.message.contains("division by zero"));

        assert_eq!(image.revision(), 1);
        assert!(image.bindings().get("y").is_none());
        assert_eq!(image.bindings().get("x"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn let_returns_bound_value() {
        let mut image = RuntimeImage::new();
        let program = parse("let :answer be 40 + 2").expect("parse");
        let v = image.eval_in_tx(&program).expect("eval");
        assert_eq!(v, Value::Number(42.0));
    }

    // --- Codec for Value (Part 0): round-trip + panic-safe malformed input ---

    #[test]
    fn value_codec_round_trips_representative_numbers() {
        for n in [
            0.0,
            -0.0,
            1.0,
            -1.0,
            42.0,
            3.141_592_653_589_793,
            f64::MIN,
            f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let value = Value::Number(n);
            let encoded = value.encode();
            let decoded = Value::decode(&encoded).expect("round-trip decode should succeed");
            assert_eq!(decoded, value, "round-trip mismatch for {n}");
        }
    }

    #[test]
    fn value_codec_round_trips_nan_bitwise() {
        // NaN != NaN, so compare the underlying bits rather than the Values.
        let encoded = Value::Number(f64::NAN).encode();
        match Value::decode(&encoded).expect("NaN round-trip decode should succeed") {
            Value::Number(n) => assert!(n.is_nan(), "expected NaN back, got {n}"),
        }
    }

    #[test]
    fn value_codec_encodes_with_version_tag_and_length() {
        let encoded = Value::Number(7.0).encode();
        assert_eq!(encoded.len(), 1 + 8, "expected 1-byte tag + 8-byte payload");
        assert_eq!(encoded[0], CODEC_VERSION_NUMBER);
    }

    #[test]
    fn value_decode_does_not_panic_on_empty_slice() {
        let err = Value::decode(&[]).expect_err("empty slice must be rejected, not panic");
        assert!(!err.0.is_empty(), "decode error must carry a message");
    }

    #[test]
    fn value_decode_does_not_panic_on_single_byte_slice() {
        // Only the version tag, no f64 payload at all.
        Value::decode(&[CODEC_VERSION_NUMBER])
            .expect_err("version-tag-only slice must be rejected, not panic");
    }

    #[test]
    fn value_decode_rejects_truncated_payload() {
        // Valid tag, only 3 payload bytes instead of 8.
        Value::decode(&[CODEC_VERSION_NUMBER, 1, 2, 3])
            .expect_err("truncated payload must be rejected");
    }

    #[test]
    fn value_decode_rejects_oversized_payload() {
        // Valid tag, too many payload bytes.
        Value::decode(&[CODEC_VERSION_NUMBER, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            .expect_err("oversized payload must be rejected");
    }

    #[test]
    fn value_decode_rejects_unknown_version_tag() {
        let err = Value::decode(&[0xFF, 0, 0, 0, 0, 0, 0, 0, 0])
            .expect_err("unknown version tag must be rejected");
        assert!(
            err.0.contains("255"),
            "expected the unknown tag in the message, got: {}",
            err.0
        );
    }
}
