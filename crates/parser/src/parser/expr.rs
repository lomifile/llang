use super::Parser;
use crate::error::ParserError;
use ast::statement::{Expression, ExpressionKind};
use token::{
    keywords::{Keyword, Operator, Punct},
    token::{Span, TokenKind},
};

impl Parser {
    pub(super) fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_logic_or()
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

        self.parse_postfix()
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
                self.expect_closing(
                    &TokenKind::Punct(Punct::RParen),
                    &TokenKind::Punct(Punct::LParen),
                    span,
                )?;
                Ok(inner)
            }

            TokenKind::Punct(Punct::LBracket) => {
                self.advance();

                let mut elements = Vec::new();
                if !self.check(&TokenKind::Punct(Punct::RBracket)) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.check(&TokenKind::Punct(Punct::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }

                let close = self.expect_closing(
                    &TokenKind::Punct(Punct::RBracket),
                    &TokenKind::Punct(Punct::LBracket),
                    span,
                )?;
                Ok(Expression {
                    kind: ExpressionKind::ArrayLiteral(elements),
                    span: Span::merge(span, close.span),
                })
            }

            TokenKind::Punct(Punct::LBrace) => {
                self.advance();

                let mut pairs = Vec::new();
                if !self.check(&TokenKind::Punct(Punct::RBrace)) {
                    loop {
                        let key_token = self.advance();
                        let key = match key_token.kind {
                            TokenKind::Identifier(s) | TokenKind::Str(s) => s,
                            other => {
                                return Err(ParserError {
                                    message: format!("expected: an object key, got: {other}"),
                                    span: key_token.span,
                                });
                            }
                        };

                        self.expect(&TokenKind::Punct(Punct::Colon))?;
                        pairs.push((key, self.parse_expression()?));

                        if self.check(&TokenKind::Punct(Punct::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }

                let close = self.expect_closing(
                    &TokenKind::Punct(Punct::RBrace),
                    &TokenKind::Punct(Punct::LBrace),
                    span,
                )?;
                Ok(Expression {
                    kind: ExpressionKind::ObjectLiteral(pairs),
                    span: Span::merge(span, close.span),
                })
            }

            _ => {
                let current = self.peek();
                let error_message = format!("expected: an expression, got: {}", current.kind);
                Err(ParserError {
                    message: error_message,
                    span: current.span,
                })
            }
        }
    }
}
