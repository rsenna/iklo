// HashMap used only in the public bindings() return type — internal storage lives behind the Substrate trait.
use std::collections::HashMap;

use iklo_ast::{BinOp, Expr, Program, Spanned};
use iklo_substrate::{Substrate, SubstrateError, Transaction};

type IkloSubstrate = iklo_substrate::memory::InMemorySubstrate<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
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

#[derive(Debug, Clone)]
pub struct RuntimeImage {
    substrate: IkloSubstrate,
}

impl Default for RuntimeImage {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeImage {
    pub fn new() -> Self {
        Self {
            substrate: IkloSubstrate::new(),
        }
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
}
