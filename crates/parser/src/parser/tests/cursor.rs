use super::{num, op, parser, span};
use token::keywords::{Keyword, Operator, Punct};
use token::token::TokenKind;

#[test]
fn expect_consumes_matching_token() {
    let mut p = parser(vec![
        TokenKind::Keyword(Keyword::Let),
        TokenKind::Identifier("x".to_string()),
    ]);

    let token = p
        .expect(&TokenKind::Keyword(Keyword::Let))
        .expect("let should match");

    assert_eq!(token.kind, TokenKind::Keyword(Keyword::Let));
    assert_eq!(token.span, span(1, 1));
    assert_eq!(p.peek().kind, TokenKind::Identifier("x".to_string()));
}

#[test]
fn expect_errors_without_consuming_on_mismatch() {
    let mut p = parser(vec![TokenKind::Punct(Punct::LParen)]);

    let error = p
        .expect(&TokenKind::Punct(Punct::RParen))
        .expect_err("rparen should not match lparen");

    assert_eq!(
        error.message,
        "expected: punctuation `)`, got: punctuation `(`"
    );
    assert_eq!(error.span, span(1, 1));
    assert_eq!(p.peek().kind, TokenKind::Punct(Punct::LParen));
}

#[test]
fn expect_matches_eof() {
    let mut p = parser(vec![]);

    assert!(p.expect(&TokenKind::Eof).is_ok());
}

#[test]
fn peek_and_advance_walk_the_token_stream() {
    let mut p = parser(vec![num(1.0), op(Operator::Plus), num(2.0)]);

    assert_eq!(p.peek().kind, num(1.0));
    assert_eq!(p.advance().kind, num(1.0));
    assert_eq!(p.advance().kind, op(Operator::Plus));
    assert_eq!(p.peek().kind, num(2.0));
    assert_eq!(p.advance().span, span(3, 1));
    assert_eq!(p.peek().kind, TokenKind::Eof);
}

#[test]
fn check_compares_current_token_without_consuming() {
    let p = parser(vec![num(1.0)]);

    assert!(p.check(&num(1.0)));
    assert!(!p.check(&num(2.0)));
    assert!(p.check(&num(1.0)));
}

#[test]
fn is_at_end_only_true_on_eof() {
    let mut p = parser(vec![num(1.0)]);

    assert!(!p.is_at_end());
    p.advance();
    assert!(p.is_at_end());
}
