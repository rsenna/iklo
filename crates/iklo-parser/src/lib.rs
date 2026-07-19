use iklo_ast::Program;
use lalrpop_util::lalrpop_mod;

mod token;
lalrpop_mod!(grammar, "/grammar.rs");

#[derive(Debug)]
pub enum ParseError {
    Lex(token::LexicalError),
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

impl From<token::LexicalError> for ParseError {
    fn from(value: token::LexicalError) -> Self {
        Self::Lex(value)
    }
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let token_stream = token::TokenStream::new(source)?;
    let parser = grammar::ProgramParser::new();
    parser.parse(token_stream).map_err(|e| match e {
        lalrpop_util::ParseError::InvalidToken { .. } => ParseError::UnexpectedEof,
        lalrpop_util::ParseError::UnrecognizedEof { .. } => ParseError::UnexpectedEof,
        lalrpop_util::ParseError::UnrecognizedToken {
            token: (_start, tok, _end),
            expected,
        } => ParseError::Unexpected {
            found: format!("{:?}", tok),
            expected: expected.join(", "),
            line: 0,
            col: 0,
        },
        lalrpop_util::ParseError::ExtraToken {
            token: (_start, tok, _end),
        } => ParseError::Unexpected {
            found: format!("{:?}", tok),
            expected: "end of input".to_string(),
            line: 0,
            col: 0,
        },
        lalrpop_util::ParseError::User { error } => ParseError::Lex(error),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iklo_ast::Expr;

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
        let src = "1 + 2\n* 3";
        assert!(parse(src).is_err());
    }
}
