use iklo_ast::{BinOp, Expr, Program, Span, Spanned};
use iklo_lexer::{tokenize, LexError, Lexeme, LexemeKind};

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
    let lexemes = tokenize(source)?;
    Parser::new(lexemes).parse_program()
}

struct Parser {
    lexemes: Vec<Lexeme>,
    pos: usize,
    paren_depth: usize,
}

impl Parser {
    fn new(lexemes: Vec<Lexeme>) -> Self {
        Self {
            lexemes,
            pos: 0,
            paren_depth: 0,
        }
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut out = Vec::new();
        self.skip_separators();
        while !self.is_eof() {
            out.push(self.parse_expr(0)?);
            if !self.at_separator() && !self.is_eof() {
                return Err(self.err_here("';', newline, or end of input"));
            }
            self.skip_separators();
        }
        Ok(out)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Spanned<Expr>, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Inside parens, newlines are noise; outside, they terminate.
            if self.paren_depth > 0 {
                self.skip_newlines();
            }
            let Some(op_lex) = self.peek().cloned() else {
                break;
            };
            let Some((l_bp, r_bp, op)) = infix_binding_power(&op_lex.kind) else {
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
        self.skip_newlines();
        let lex = self.advance_lexeme()?;
        match lex.kind {
            LexemeKind::Number(n) => Ok(Spanned::new(Expr::Number(n), lex.span)),
            LexemeKind::ColonName(name) => Ok(Spanned::new(Expr::LexRef(name), lex.span)),
            LexemeKind::Let => self.parse_let(lex.span),
            LexemeKind::LParen => {
                self.paren_depth += 1;
                let expr = self.parse_expr(0)?;
                self.skip_newlines();
                self.expect(|k| matches!(k, LexemeKind::RParen), "')'")?;
                self.paren_depth -= 1;
                Ok(expr)
            }
            other => Err(self.err_expected_lexeme(other, "expression")),
        }
    }

    fn parse_let(&mut self, let_span: Span) -> Result<Spanned<Expr>, ParseError> {
        self.skip_newlines();
        let name = match self.advance_kind()? {
            LexemeKind::ColonName(s) => s,
            other => {
                return Err(self.err_expected_lexeme(other, "':name' after 'let'"));
            }
        };
        self.skip_newlines();
        self.expect(|k| matches!(k, LexemeKind::Be), "'be'")?;
        let value = self.parse_expr(0)?;
        let span = Span::new(
            let_span.start,
            value.span.end,
            let_span.line,
            let_span.col,
        );
        Ok(Spanned::new(
            Expr::Let {
                name,
                value: Box::new(value),
            },
            span,
        ))
    }

    fn expect<F>(&mut self, pred: F, expected: &str) -> Result<(), ParseError>
    where
        F: Fn(&LexemeKind) -> bool,
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
        F: Fn(&LexemeKind) -> bool,
    {
        self.peek().map(|t| pred(&t.kind)).unwrap_or(false)
    }

    fn peek(&self) -> Option<&Lexeme> {
        self.lexemes.get(self.pos)
    }

    fn advance(&mut self) -> Option<Lexeme> {
        let lex = self.lexemes.get(self.pos).cloned();
        if lex.is_some() {
            self.pos += 1;
        }
        lex
    }

    fn advance_lexeme(&mut self) -> Result<Lexeme, ParseError> {
        self.advance().ok_or(ParseError::UnexpectedEof)
    }

    fn advance_kind(&mut self) -> Result<LexemeKind, ParseError> {
        Ok(self.advance_lexeme()?.kind)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.lexemes.len()
    }

    fn skip_newlines(&mut self) {
        while self.match_kind(|k| matches!(k, LexemeKind::Newline)) {
            self.pos += 1;
        }
    }

    fn at_separator(&self) -> bool {
        self.match_kind(|k| matches!(k, LexemeKind::Newline | LexemeKind::Semi))
    }

    fn skip_separators(&mut self) {
        while self.at_separator() {
            self.pos += 1;
        }
    }

    fn err_here(&self, expected: &str) -> ParseError {
        if let Some(lex) = self.peek() {
            ParseError::Unexpected {
                found: format!("{:?}", lex.kind),
                expected: expected.to_string(),
                line: lex.span.line,
                col: lex.span.col,
            }
        } else {
            ParseError::UnexpectedEof
        }
    }

    fn err_expected_lexeme(&self, found: LexemeKind, expected: &str) -> ParseError {
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

fn infix_binding_power(kind: &LexemeKind) -> Option<(u8, u8, BinOp)> {
    match kind {
        LexemeKind::Plus => Some((1, 2, BinOp::Add)),
        LexemeKind::Minus => Some((1, 2, BinOp::Sub)),
        LexemeKind::Star => Some((3, 4, BinOp::Mul)),
        LexemeKind::Slash => Some((3, 4, BinOp::Div)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_let_and_ref() {
        let src = "let :x be 1 + 2 * 3; :x";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn let_is_an_expression() {
        let src = "let :x be 40 + 2";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 1);
        matches!(program[0].node, Expr::Let { .. });
    }

    #[test]
    fn colon_name_reads_binding() {
        let src = ":answer";
        let program = parse(src).expect("parse");
        matches!(program[0].node, Expr::LexRef(_));
    }

    #[test]
    fn newline_terminates_when_expression_is_valid() {
        let src = "let :x be 1\nlet :y be 2\n:x + :y";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 3);
    }

    #[test]
    fn trailing_operator_continues_expression_across_newline() {
        let src = "let :x be 1 +\n  2";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn newline_after_let_continues_to_name() {
        let src = "let\n  :x be\n  40 + 2";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn parens_swallow_newlines() {
        let src = "(1 +\n 2\n * 3)";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn semicolon_forces_termination() {
        let src = "let :x be 1; :x";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn multiple_blank_lines_between_expressions() {
        let src = "\n\nlet :x be 1\n\n\n:x\n";
        let program = parse(src).expect("parse");
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn missing_operator_across_newline_is_two_expressions() {
        // `1 + 2` is valid, so newline ends it; `* 3` then fails to parse.
        let src = "1 + 2\n* 3";
        assert!(parse(src).is_err());
    }
}

