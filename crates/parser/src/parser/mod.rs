mod decl;
mod expr;
mod stmt;
mod types;

#[cfg(test)]
mod tests;

use crate::error::ParserError;
use token::token::{Span, Token, TokenKind};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) -> Token {
        let current = self.tokens[self.position].clone();
        self.position += 1;
        current
    }

    fn check(&self, match_to: &TokenKind) -> bool {
        &self.tokens[self.position].kind == match_to
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.position].kind == TokenKind::Eof
    }

    fn expect(&mut self, match_to: &TokenKind) -> Result<Token, ParserError> {
        let current = &self.tokens[self.position];
        if &current.kind == match_to {
            return Ok(self.advance());
        }
        let error_message = format!("expected: {}, got: {}", match_to, current.kind);
        Err(ParserError {
            message: error_message,
            span: current.span,
        })
    }

    fn expect_closing(
        &mut self,
        close: &TokenKind,
        open: &TokenKind,
        open_span: Span,
    ) -> Result<Token, ParserError> {
        let current = &self.tokens[self.position];
        if &current.kind == close {
            return Ok(self.advance());
        }
        let error_message = format!(
            "expected: {close} to close {open} opened at {open_span}, got: {}",
            current.kind
        );
        Err(ParserError {
            message: error_message,
            span: current.span,
        })
    }

    fn expect_identifier(&mut self, message: &str) -> Result<String, ParserError> {
        let token = self.advance();
        match token.kind {
            TokenKind::Identifier(s) => Ok(s),
            _ => Err(ParserError {
                span: token.span,
                message: message.to_string(),
            }),
        }
    }
}
