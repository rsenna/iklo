use std::collections::HashMap;

use iklo_ast::{BinOp, Expr, Program, Spanned, Stmt};

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

#[derive(Debug, Clone)]
pub struct RuntimeImage {
    bindings: HashMap<String, Value>,
    revision: u64,
}

impl Default for RuntimeImage {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeImage {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            revision: 0,
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }

    pub fn eval_in_tx(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut tx = Transaction::from_image(self);
        let result = tx.eval_program(program)?;
        tx.commit(self);
        Ok(result)
    }
}

struct Transaction {
    bindings: HashMap<String, Value>,
}

impl Transaction {
    fn from_image(image: &RuntimeImage) -> Self {
        Self {
            bindings: image.bindings.clone(),
        }
    }

    fn commit(self, image: &mut RuntimeImage) {
        image.bindings = self.bindings;
        image.revision += 1;
    }

    fn eval_program(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last = Value::Number(0.0);
        for stmt in program {
            last = self.eval_stmt(stmt)?;
        }
        Ok(last)
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Let { name, expr } => {
                let value = self.eval_expr(expr)?;
                self.bindings.insert(name.clone(), value.clone());
                Ok(value)
            }
            Stmt::Expr(expr) => self.eval_expr(expr),
        }
    }

    fn eval_expr(&mut self, expr: &Spanned<Expr>) -> Result<Value, RuntimeError> {
        match &expr.node {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::TokenRef(name) => self
                .bindings
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined token '{name}'"))),
            Expr::Binary { op, left, right } => {
                let left = self.eval_expr(left)?;
                let right = self.eval_expr(right)?;
                self.eval_binop(*op, left, right)
            }
        }
    }

    fn eval_binop(&self, op: BinOp, left: Value, right: Value) -> Result<Value, RuntimeError> {
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
}

#[cfg(test)]
mod tests {
    use iklo_parser::parse;

    use super::*;

    #[test]
    fn rollback_keeps_image_unchanged() {
        let mut image = RuntimeImage::new();
        let setup = parse("let x = 10").expect("parse");
        image.eval_in_tx(&setup).expect("eval");

        let bad = parse("let y = x / 0").expect("parse");
        let err = image.eval_in_tx(&bad).expect_err("must fail");
        assert!(err.message.contains("division by zero"));

        assert_eq!(image.revision(), 1);
        assert!(image.bindings().get("y").is_none());
        assert_eq!(image.bindings().get("x"), Some(&Value::Number(10.0)));
    }
}

