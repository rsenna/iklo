use iklo_ast::{BinOp, Expr, Program, Span, Spanned, Stmt};
use iklo_lexer::{tokenize, LexError, Token, TokenKind};

#[derive(Debug)]
pub enum ParseError {
    Lex(LexError),
    UnexpectedEof,
    Unexpected {
        found: String,
        expected: String,
        line: usize,
        col: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(e) => write!(f, "{e}"),
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::Unexpected {
                found,
                expected,
                line,
                col,
            } => write!(
                f,
                "parse error at {}:{}: expected {}, found {}",
                line, col, expected, found
            ),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = tokenize(source)?;
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut out = Vec::new();
        while !self.is_eof() {
            out.push(self.parse_stmt()?);
            if self.match_kind(|k| matches!(k, TokenKind::Semi)) {
                self.advance();
                while self.match_kind(|k| matches!(k, TokenKind::Semi)) {
                    self.advance();
                }
            } else if !self.is_eof() {
                return Err(self.err_here("';' or end of input"));
            }
        }
        Ok(out)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kind(|k| matches!(k, TokenKind::Let)) {
            self.advance();
            let name = match self.advance_kind()? {
                TokenKind::Ident(s) => s,
                other => return Err(self.err_expected_token(other, "identifier")),
            };
            self.expect(|k| matches!(k, TokenKind::Eq), "'='")?;
            let expr = self.parse_expr(0)?;
            return Ok(Stmt::Let { name, expr });
        }

        Ok(Stmt::Expr(self.parse_expr(0)?))
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Spanned<Expr>, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            let Some(op_tok) = self.peek().cloned() else {
                break;
            };
            let Some((l_bp, r_bp, op)) = infix_binding_power(&op_tok.kind) else {
                break;
            };
            if l_bp < min_bp {
                break;
            }

            self.advance();
            let rhs = self.parse_expr(r_bp)?;
            let span = Span::new(
                lhs.span.start,
                rhs.span.end,
                lhs.span.line,
                lhs.span.col,
            );
            lhs = Spanned::new(
                Expr::Binary {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                },
                span,
            );
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let tok = self.advance_token()?;
        match tok.kind {
            TokenKind::Number(n) => Ok(Spanned::new(Expr::Number(n), tok.span)),
            TokenKind::Ident(name) => Ok(Spanned::new(Expr::TokenRef(name), tok.span)),
            TokenKind::LParen => {
                let expr = self.parse_expr(0)?;
                self.expect(|k| matches!(k, TokenKind::RParen), "')'")?;
                Ok(expr)
            }
            other => Err(self.err_expected_token(other, "expression")),
        }
    }

    fn expect<F>(&mut self, pred: F, expected: &str) -> Result<(), ParseError>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if self.match_kind(pred) {
            self.advance();
            Ok(())
        } else {
            Err(self.err_here(expected))
        }
    }

    fn match_kind<F>(&self, pred: F) -> bool
    where
        F: Fn(&TokenKind) -> bool,
    {
        self.peek().map(|t| pred(&t.kind)).unwrap_or(false)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn advance_token(&mut self) -> Result<Token, ParseError> {
        self.advance().ok_or(ParseError::UnexpectedEof)
    }

    fn advance_kind(&mut self) -> Result<TokenKind, ParseError> {
        Ok(self.advance_token()?.kind)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn err_here(&self, expected: &str) -> ParseError {
        if let Some(tok) = self.peek() {
            ParseError::Unexpected {
                found: format!("{:?}", tok.kind),
                expected: expected.to_string(),
                line: tok.span.line,
                col: tok.span.col,
            }
        } else {
            ParseError::UnexpectedEof
        }
    }

    fn err_expected_token(&self, found: TokenKind, expected: &str) -> ParseError {
        let (line, col) = self
            .peek()
            .map(|t| (t.span.line, t.span.col))
            .unwrap_or((0, 0));
        ParseError::Unexpected {
            found: format!("{:?}", found),
            expected: expected.to_string(),
            line,
            col,
        }
    }
}

fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8, BinOp)> {
    match kind {
        TokenKind::Plus => Some((1, 2, BinOp::Add)),
        TokenKind::Minus => Some((1, 2, BinOp::Sub)),
        TokenKind::Star => Some((3, 4, BinOp::Mul)),
        TokenKind::Slash => Some((3, 4, BinOp::Div)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_let_and_expr() {
        let src = "let x = 1 + 2 * 3; x";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 2);
    }
}

