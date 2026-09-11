use super::Parser;
use crate::error::ParserError;
use ast::statement::{Declaration, DeclarationKind, Statement, StatementKind};
use lexer::lexer::Lexer;
use token::keywords::Operator;
use token::token::{Span, Token, TokenKind};

mod cursor;
mod decl;
mod expr;
mod program;
mod stmt;
mod types;

fn span(col: u32, len: u32) -> Span {
    Span { line: 1, col, len }
}

fn at(line: u32, col: u32, len: u32) -> Span {
    Span { line, col, len }
}

fn parser(kinds: Vec<TokenKind>) -> Parser {
    let mut col = 1;
    let mut tokens: Vec<Token> = kinds
        .into_iter()
        .map(|kind| {
            let token = Token::new(kind, span(col, 1));
            col += 1;
            token
        })
        .collect();
    tokens.push(Token::new(TokenKind::Eof, span(col, 0)));
    Parser::new(tokens)
}

fn num(n: f64) -> TokenKind {
    TokenKind::Number(n)
}

fn op(o: Operator) -> TokenKind {
    TokenKind::Operator(o)
}

fn source(src: &str) -> Parser {
    let tokens = Lexer::new(src).tokenize().expect("source should lex");
    Parser::new(tokens)
}

fn program(src: &str) -> Vec<Declaration> {
    source(src)
        .parse_program()
        .unwrap_or_else(|e| panic!("`{src}` should parse, got {}: {}", e.span, e.message))
}

fn declaration(src: &str) -> Declaration {
    let mut declarations = program(src);
    assert_eq!(
        declarations.len(),
        1,
        "`{src}` should be exactly one declaration"
    );
    declarations.remove(0)
}

fn declaration_kind(src: &str) -> DeclarationKind {
    declaration(src).kind
}

fn statement(src: &str) -> Statement {
    match declaration(src).kind {
        DeclarationKind::Statement(s) => s,
        other => panic!("`{src}` should be a statement, got {other:?}"),
    }
}

fn statement_kind(src: &str) -> StatementKind {
    statement(src).kind
}

fn error(src: &str) -> ParserError {
    source(src)
        .parse_program()
        .expect_err(&format!("`{src}` should not parse"))
}
