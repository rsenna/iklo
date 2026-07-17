use iklo_ast::Span;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"#[^\n]*")]
pub enum LexemeKind {
    #[token("let")]
    Let,
    #[token("be")]
    Be,

    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    #[regex(r":[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice()[1..].to_string())]
    ColonName(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme {
    pub kind: LexemeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.message, self.span.line, self.span.col
        )
    }
}

impl std::error::Error for LexError {}

fn line_col_at(src: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for ch in src[..byte_offset].chars() {
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn tokenize(src: &str) -> Result<Vec<Lexeme>, LexError> {
    let mut lexer = LexemeKind::lexer(src);
    let mut out = Vec::new();

    while let Some(next) = lexer.next() {
        let range = lexer.span();
        let (line, col) = line_col_at(src, range.start);
        let span = Span::new(range.start, range.end, line, col);

        match next {
            Ok(kind) => out.push(Lexeme { kind, span }),
            Err(_) => {
                return Err(LexError {
                    message: "invalid token".to_string(),
                    span,
                });
            }
        }
    }

    Ok(out)
}

