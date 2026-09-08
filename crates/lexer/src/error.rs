use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    UnterminatedString { line: u32, col: u32 },
    InvalidNumber { line: u32, col: u32, text: String },
    UnexpectedChar { line: u32, col: u32, ch: char },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedString { line, col } => write!(
                f,
                "{line}:{col}: unterminated string literal: the string opened here is never closed"
            ),
            Self::InvalidNumber { line, col, text } => write!(
                f,
                "{line}:{col}: invalid number literal `{text}`: expected digits with at most one decimal point"
            ),
            Self::UnexpectedChar { line, col, ch } => write!(
                f,
                "{line}:{col}: unexpected character `{ch}`: no token can start with it"
            ),
        }
    }
}

impl std::error::Error for LexerError {}
