use iklo_lexer::{tokenize, LexError, LexemeKind};

/// Source position used as the LALRPOP `Location` type. Carries the byte
/// offset alongside the lexer's real line/column so spans and parse errors
/// keep accurate coordinates.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Token {
    T_Let,
    T_Be,
    T_Number(f64),
    T_ColonName(String),
    T_Ident(String),
    T_Eq,
    T_Plus,
    T_Minus,
    T_Star,
    T_Slash,
    T_LParen,
    T_RParen,
    T_Semi,
    T_Newline,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::T_Let, Token::T_Let) => true,
            (Token::T_Be, Token::T_Be) => true,
            (Token::T_Number(a), Token::T_Number(b)) => a == b,
            (Token::T_ColonName(a), Token::T_ColonName(b)) => a == b,
            (Token::T_Ident(a), Token::T_Ident(b)) => a == b,
            (Token::T_Eq, Token::T_Eq) => true,
            (Token::T_Plus, Token::T_Plus) => true,
            (Token::T_Minus, Token::T_Minus) => true,
            (Token::T_Star, Token::T_Star) => true,
            (Token::T_Slash, Token::T_Slash) => true,
            (Token::T_LParen, Token::T_LParen) => true,
            (Token::T_RParen, Token::T_RParen) => true,
            (Token::T_Semi, Token::T_Semi) => true,
            (Token::T_Newline, Token::T_Newline) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::T_Let => write!(f, "let"),
            Token::T_Be => write!(f, "be"),
            Token::T_Number(n) => write!(f, "{n}"),
            Token::T_ColonName(n) => write!(f, ":{n}"),
            Token::T_Ident(n) => write!(f, "{n}"),
            Token::T_Eq => write!(f, "="),
            Token::T_Plus => write!(f, "+"),
            Token::T_Minus => write!(f, "-"),
            Token::T_Star => write!(f, "*"),
            Token::T_Slash => write!(f, "/"),
            Token::T_LParen => write!(f, "("),
            Token::T_RParen => write!(f, ")"),
            Token::T_Semi => write!(f, ";"),
            Token::T_Newline => write!(f, "newline"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LexicalError {
    InvalidToken { start: usize, end: usize },
    LexError(LexError),
}

impl std::fmt::Display for LexicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexicalError::InvalidToken { start, end } => {
                write!(f, "invalid token at bytes {start}..{end}")
            }
            LexicalError::LexError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LexicalError {}

impl From<LexError> for LexicalError {
    fn from(e: LexError) -> Self {
        LexicalError::LexError(e)
    }
}

fn convert_kind(kind: &LexemeKind) -> Option<Token> {
    match kind {
        LexemeKind::Let => Some(Token::T_Let),
        LexemeKind::Be => Some(Token::T_Be),
        LexemeKind::Number(n) => Some(Token::T_Number(*n)),
        LexemeKind::ColonName(s) => Some(Token::T_ColonName(s.clone())),
        LexemeKind::Ident(s) => Some(Token::T_Ident(s.clone())),
        LexemeKind::Eq => Some(Token::T_Eq),
        LexemeKind::Plus => Some(Token::T_Plus),
        LexemeKind::Minus => Some(Token::T_Minus),
        LexemeKind::Star => Some(Token::T_Star),
        LexemeKind::Slash => Some(Token::T_Slash),
        LexemeKind::LParen => Some(Token::T_LParen),
        LexemeKind::RParen => Some(Token::T_RParen),
        LexemeKind::Semi => Some(Token::T_Semi),
        LexemeKind::Newline => Some(Token::T_Newline),
    }
}

fn should_drop_newline(prev: &Token, next: &Token) -> bool {
    matches!(
        prev,
        Token::T_Plus | Token::T_Minus | Token::T_Star | Token::T_Slash | Token::T_Let | Token::T_Be
    ) || (matches!(prev, Token::T_ColonName(_)) && matches!(next, Token::T_Be))
}

pub struct TokenStream {
    lexemes: Vec<iklo_lexer::Lexeme>,
    pos: usize,
    prev: Token,
    paren_depth: usize,
}

impl TokenStream {
    pub fn new(src: &str) -> Result<Self, LexicalError> {
        let lexemes = tokenize(src)?;
        Ok(Self {
            lexemes,
            pos: 0,
            prev: Token::T_Semi,
            paren_depth: 0,
        })
    }
}

impl Iterator for TokenStream {
    type Item = Result<(Position, Token, Position), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let lex = self.lexemes.get(self.pos)?;
            let token = convert_kind(&lex.kind)?;
            let start = Position {
                offset: lex.span.start,
                line: lex.span.line,
                col: lex.span.col,
            };
            let end = Position {
                offset: lex.span.end,
                line: lex.span.line,
                col: lex.span.col,
            };
            self.pos += 1;

            match &token {
                Token::T_LParen => self.paren_depth += 1,
                Token::T_RParen => {
                    if self.paren_depth > 0 {
                        self.paren_depth -= 1;
                    }
                }
                _ => {}
            }

            if matches!(token, Token::T_Newline) {
                let next_kind = self.lexemes.get(self.pos).and_then(|l| convert_kind(&l.kind));
                let next_token = next_kind.as_ref().unwrap_or(&Token::T_Semi);

                if self.paren_depth > 0 || should_drop_newline(&self.prev, next_token) {
                    continue;
                }
            }

            let prev = token.clone();
            let result = Ok((start, token, end));
            self.prev = prev;
            return Some(result);
        }
    }
}
