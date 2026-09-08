use crate::keywords;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(keywords::Keyword),
    Operator(keywords::Operator),
    Punct(keywords::Punct),
    Identifier(String),
    Number(f64),
    Str(String),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub col: u32,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyword(kw) => write!(f, "keyword `{kw}`"),
            Self::Operator(op) => write!(f, "operator `{op}`"),
            Self::Punct(p) => write!(f, "punctuation `{p}`"),
            Self::Identifier(name) => write!(f, "identifier `{name}`"),
            Self::Number(n) => write!(f, "number `{n}`"),
            Self::Str(s) => write!(f, "string `\"{s}\"`"),
            Self::Eof => f.write_str("end of input"),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}
