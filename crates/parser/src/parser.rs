use std::error::Error;

use crate::error::ParserError;
use ast::statement::{Expression, ExpressionKind, Statement, StatementKind};
use token::{
    keywords::{Keyword, Operator, Punct},
    token::{Span, Token, TokenKind},
};

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

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        let start = self.tokens[self.position].clone();
        self.advance();

        let value = if self.tokens[self.position].kind == TokenKind::Punct(Punct::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let _ = self.expect(&TokenKind::Punct(Punct::Semicolon));

        Ok(Statement {
            kind: StatementKind::Return(value),
            span: Span::merge(start.span, self.tokens[self.position].span),
        })
    }

    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_comparison()?;
        while let TokenKind::Operator(op @ (Operator::Eq | Operator::NotEq)) = self.peek().kind {
            self.advance();
            let right = self.parse_comparison()?;
            let span = Span::merge(left.span, right.span);
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_term()?;
        while let TokenKind::Operator(
            op @ (Operator::GreaterThan
            | Operator::LessThan
            | Operator::GreaterOrEqual
            | Operator::LessOrEqual),
        ) = self.peek().kind
        {
            self.advance();
            let right = self.parse_term()?;
            let span = Span::merge(left.span, right.span);
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_factor()?;
        while let TokenKind::Operator(op @ (Operator::Plus | Operator::Minus)) = self.peek().kind {
            self.advance();
            let right = self.parse_factor()?;
            let span = Span::merge(left.span, right.span);
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_unary()?;
        while let TokenKind::Operator(
            op @ (Operator::Multiply | Operator::Divide | Operator::Modulo),
        ) = self.peek().kind
        {
            self.advance();
            let right = self.parse_unary()?;
            let span = Span::merge(left.span, right.span);
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParserError> {
        let start = self.peek().span;
        if let TokenKind::Operator(op @ (Operator::Not | Operator::Minus)) = self.peek().kind {
            self.advance();
            let operand = self.parse_unary()?;
            let span = Span::merge(start, operand.span);
            return Ok(Expression {
                kind: ExpressionKind::UnaryOp {
                    op,
                    operand: Box::new(operand),
                },
                span,
            });
        }
        self.parse_primary()
    }

    fn parse_logic_and(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_equality()?;
        while let TokenKind::Operator(op @ Operator::And) = self.peek().kind {
            self.advance();
            let right = self.parse_equality()?;
            let span = Span::merge(left.span, right.span);
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_logic_or(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_logic_and()?;

        while let TokenKind::Operator(op @ Operator::Or) = self.peek().kind {
            self.advance();
            let right = self.parse_logic_and()?;
            let span = Span::merge(left.span, right.span);
            left = Expression {
                kind: ExpressionKind::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match &self.peek().kind {
            TokenKind::Keyword(Keyword::If) => todo!(),
            TokenKind::Keyword(Keyword::While) => todo!(),
            TokenKind::Keyword(Keyword::For) => todo!(),
            TokenKind::Keyword(Keyword::Return) => todo!(),
            TokenKind::Punct(Punct::LBrace) => todo!(),
            TokenKind::Operator(Operator::Increment) => todo!(),
            TokenKind::Operator(Operator::Decrement) => todo!(),
            _ => todo!(),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_logic_or()
    }

    pub fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        let span = self.peek().span;

        match &self.peek().kind {
            TokenKind::Str(_)
            | TokenKind::Number(_)
            | TokenKind::Identifier(_)
            | TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::Null) => {
                let token = self.advance();
                let kind = match token.kind {
                    TokenKind::Str(s) => ExpressionKind::LiteralString(s),
                    TokenKind::Number(n) => ExpressionKind::LiteralNumber(n),
                    TokenKind::Identifier(name) => ExpressionKind::Identifier(name),
                    TokenKind::Keyword(Keyword::True) => ExpressionKind::LiteralBoolean(true),
                    TokenKind::Keyword(Keyword::False) => ExpressionKind::LiteralBoolean(false),
                    TokenKind::Keyword(Keyword::Null) => ExpressionKind::LiteralNull,
                    _ => unreachable!(),
                };
                Ok(Expression { kind, span })
            }

            TokenKind::Punct(Punct::LParen) => {
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::Punct(Punct::RParen))?;
                Ok(inner)
            }

            _ => {
                let current = self.peek();
                let error_message = format!("cannot parse: {} on {}", current.kind, current.span);
                Err(ParserError {
                    message: error_message,
                    span: current.span,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use ast::statement::{Expression, ExpressionKind};
    use token::keywords::{Keyword, Operator, Punct};
    use token::token::{Span, Token, TokenKind};

    fn span(col: u32, len: u32) -> Span {
        Span { line: 1, col, len }
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

    fn expr(kinds: Vec<TokenKind>) -> Expression {
        parser(kinds)
            .parse_expression()
            .expect("expression should parse")
    }

    fn kind(kinds: Vec<TokenKind>) -> ExpressionKind {
        expr(kinds).kind
    }

    fn lit(n: f64) -> Expression {
        Expression {
            kind: ExpressionKind::LiteralNumber(n),
            span: span(1, 1),
        }
    }

    fn bin(o: Operator, left: Expression, right: Expression) -> ExpressionKind {
        ExpressionKind::BinaryOp {
            op: o,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn unary(o: Operator, operand: Expression) -> ExpressionKind {
        ExpressionKind::UnaryOp {
            op: o,
            operand: Box::new(operand),
        }
    }

    fn at(mut e: Expression, col: u32) -> Expression {
        e.span = span(col, 1);
        e
    }

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

    #[test]
    fn parse_primary_reads_every_literal_form() {
        assert_eq!(kind(vec![num(4.5)]), ExpressionKind::LiteralNumber(4.5));
        assert_eq!(
            kind(vec![TokenKind::Str("hi".to_string())]),
            ExpressionKind::LiteralString("hi".to_string())
        );
        assert_eq!(
            kind(vec![TokenKind::Identifier("x".to_string())]),
            ExpressionKind::Identifier("x".to_string())
        );
        assert_eq!(
            kind(vec![TokenKind::Keyword(Keyword::True)]),
            ExpressionKind::LiteralBoolean(true)
        );
        assert_eq!(
            kind(vec![TokenKind::Keyword(Keyword::False)]),
            ExpressionKind::LiteralBoolean(false)
        );
        assert_eq!(
            kind(vec![TokenKind::Keyword(Keyword::Null)]),
            ExpressionKind::LiteralNull
        );
    }

    #[test]
    fn parse_primary_carries_the_token_span() {
        let mut p = parser(vec![op(Operator::Minus), num(1.0)]);
        p.advance();

        let parsed = p.parse_primary().expect("number should parse");

        assert_eq!(parsed.span, span(2, 1));
    }

    #[test]
    fn parse_primary_rejects_a_non_expression_token() {
        let error = parser(vec![TokenKind::Punct(Punct::RBrace)])
            .parse_primary()
            .expect_err("a closing brace starts no expression");

        assert_eq!(error.message, "cannot parse: punctuation `}` on 1:1");
        assert_eq!(error.span, span(1, 1));
    }

    #[test]
    fn parse_unary_applies_prefix_operators_right_to_left() {
        assert_eq!(
            kind(vec![op(Operator::Minus), num(1.0)]),
            unary(Operator::Minus, at(lit(1.0), 2))
        );
        assert_eq!(
            kind(vec![op(Operator::Not), op(Operator::Not), num(1.0)]),
            unary(
                Operator::Not,
                Expression {
                    kind: unary(Operator::Not, at(lit(1.0), 3)),
                    span: span(2, 2),
                }
            )
        );
    }

    #[test]
    fn parse_unary_binds_tighter_than_factor() {
        assert_eq!(
            kind(vec![
                op(Operator::Minus),
                num(2.0),
                op(Operator::Multiply),
                num(3.0)
            ]),
            bin(
                Operator::Multiply,
                Expression {
                    kind: unary(Operator::Minus, at(lit(2.0), 2)),
                    span: span(1, 2),
                },
                at(lit(3.0), 4)
            )
        );
    }

    #[test]
    fn parse_factor_is_left_associative() {
        assert_eq!(
            kind(vec![
                num(8.0),
                op(Operator::Divide),
                num(4.0),
                op(Operator::Modulo),
                num(3.0)
            ]),
            bin(
                Operator::Modulo,
                Expression {
                    kind: bin(Operator::Divide, lit(8.0), at(lit(4.0), 3)),
                    span: span(1, 3),
                },
                at(lit(3.0), 5)
            )
        );
    }

    #[test]
    fn parse_term_is_left_associative() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::Plus),
                num(2.0),
                op(Operator::Minus),
                num(3.0)
            ]),
            bin(
                Operator::Minus,
                Expression {
                    kind: bin(Operator::Plus, lit(1.0), at(lit(2.0), 3)),
                    span: span(1, 3),
                },
                at(lit(3.0), 5)
            )
        );
    }

    #[test]
    fn parse_term_gives_factor_higher_precedence() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::Plus),
                num(2.0),
                op(Operator::Multiply),
                num(3.0)
            ]),
            bin(
                Operator::Plus,
                lit(1.0),
                Expression {
                    kind: bin(Operator::Multiply, at(lit(2.0), 3), at(lit(3.0), 5)),
                    span: span(3, 3),
                }
            )
        );
    }

    #[test]
    fn parse_comparison_sits_below_term() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::Plus),
                num(2.0),
                op(Operator::LessThan),
                num(4.0)
            ]),
            bin(
                Operator::LessThan,
                Expression {
                    kind: bin(Operator::Plus, lit(1.0), at(lit(2.0), 3)),
                    span: span(1, 3),
                },
                at(lit(4.0), 5)
            )
        );
    }

    #[test]
    fn parse_comparison_accepts_each_relational_operator() {
        for o in [
            Operator::GreaterThan,
            Operator::LessThan,
            Operator::GreaterOrEqual,
            Operator::LessOrEqual,
        ] {
            assert_eq!(
                kind(vec![num(1.0), op(o), num(2.0)]),
                bin(o, lit(1.0), at(lit(2.0), 3)),
                "{o} should parse as a comparison"
            );
        }
    }

    #[test]
    fn parse_equality_sits_below_comparison() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::LessThan),
                num(2.0),
                op(Operator::Eq),
                TokenKind::Keyword(Keyword::True)
            ]),
            bin(
                Operator::Eq,
                Expression {
                    kind: bin(Operator::LessThan, lit(1.0), at(lit(2.0), 3)),
                    span: span(1, 3),
                },
                Expression {
                    kind: ExpressionKind::LiteralBoolean(true),
                    span: span(5, 1),
                }
            )
        );
    }

    #[test]
    fn parse_equality_accepts_both_operators() {
        for o in [Operator::Eq, Operator::NotEq] {
            assert_eq!(
                kind(vec![num(1.0), op(o), num(2.0)]),
                bin(o, lit(1.0), at(lit(2.0), 3)),
                "{o} should parse as an equality"
            );
        }
    }

    #[test]
    fn parse_logic_and_sits_below_equality() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::Eq),
                num(2.0),
                op(Operator::And),
                TokenKind::Keyword(Keyword::True)
            ]),
            bin(
                Operator::And,
                Expression {
                    kind: bin(Operator::Eq, lit(1.0), at(lit(2.0), 3)),
                    span: span(1, 3),
                },
                Expression {
                    kind: ExpressionKind::LiteralBoolean(true),
                    span: span(5, 1),
                }
            )
        );
    }

    #[test]
    fn parse_logic_or_sits_below_logic_and() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::Or),
                num(2.0),
                op(Operator::And),
                num(3.0)
            ]),
            bin(
                Operator::Or,
                lit(1.0),
                Expression {
                    kind: bin(Operator::And, at(lit(2.0), 3), at(lit(3.0), 5)),
                    span: span(3, 3),
                }
            )
        );
    }

    #[test]
    fn parse_logic_or_is_left_associative() {
        assert_eq!(
            kind(vec![
                num(1.0),
                op(Operator::Or),
                num(2.0),
                op(Operator::Or),
                num(3.0)
            ]),
            bin(
                Operator::Or,
                Expression {
                    kind: bin(Operator::Or, lit(1.0), at(lit(2.0), 3)),
                    span: span(1, 3),
                },
                at(lit(3.0), 5)
            )
        );
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(
            kind(vec![
                TokenKind::Punct(Punct::LParen),
                num(1.0),
                op(Operator::Plus),
                num(2.0),
                TokenKind::Punct(Punct::RParen),
                op(Operator::Multiply),
                num(3.0)
            ]),
            bin(
                Operator::Multiply,
                Expression {
                    kind: bin(Operator::Plus, at(lit(1.0), 2), at(lit(2.0), 4)),
                    span: span(2, 3),
                },
                at(lit(3.0), 7)
            )
        );
    }

    #[test]
    fn an_unclosed_group_reports_the_missing_paren() {
        let error = parser(vec![TokenKind::Punct(Punct::LParen), num(1.0)])
            .parse_expression()
            .expect_err("the group is never closed");

        assert_eq!(
            error.message,
            "expected: punctuation `)`, got: end of input"
        );
    }

    #[test]
    fn a_missing_right_operand_reports_the_offending_token() {
        let error = parser(vec![num(1.0), op(Operator::Plus)])
            .parse_expression()
            .expect_err("plus has no right operand");

        assert_eq!(error.message, "cannot parse: end of input on 1:3");
        assert_eq!(error.span, span(3, 0));
    }

    #[test]
    fn a_binary_span_covers_both_operands() {
        assert_eq!(
            expr(vec![
                num(1.0),
                op(Operator::Plus),
                num(2.0),
                op(Operator::Multiply),
                num(3.0)
            ])
            .span,
            span(1, 5)
        );
    }

    #[test]
    fn a_unary_span_covers_the_operator_and_its_operand() {
        assert_eq!(
            expr(vec![op(Operator::Minus), op(Operator::Not), num(1.0)]).span,
            span(1, 3)
        );
    }

    #[test]
    fn a_group_span_covers_only_the_inner_expression() {
        assert_eq!(
            expr(vec![
                TokenKind::Punct(Punct::LParen),
                num(1.0),
                op(Operator::Plus),
                num(2.0),
                TokenKind::Punct(Punct::RParen)
            ])
            .span,
            span(2, 3)
        );
    }

    #[test]
    fn parse_expression_stops_at_the_first_token_it_cannot_use() {
        let mut p = parser(vec![
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            TokenKind::Punct(Punct::Semicolon),
        ]);

        p.parse_expression().expect("the sum should parse");

        assert_eq!(p.peek().kind, TokenKind::Punct(Punct::Semicolon));
    }
}
