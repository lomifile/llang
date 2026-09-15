use super::Parser;
use crate::error::ParserError;
use ast::statement::{
    Declaration, Expression, ExpressionKind, ForInit, ForStep, Statement, StatementKind,
};
use token::{
    keywords::{Keyword, Operator, Punct},
    token::{Span, TokenKind},
};

impl Parser {
    pub fn parse_program(&mut self) -> Result<Vec<Declaration>, ParserError> {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            declarations.push(self.parse_declaration()?);
        }

        Ok(declarations)
    }

    pub(super) fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match &self.peek().kind {
            TokenKind::Keyword(Keyword::If) => self.parse_if(),
            TokenKind::Keyword(Keyword::While) => self.parse_while(),
            TokenKind::Keyword(Keyword::For) => self.parse_for(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Punct(Punct::LBrace) => self.parse_block(),
            TokenKind::Operator(Operator::Increment | Operator::Decrement) => self.parse_incdec(),
            _ => self.parse_expression_or_assign(),
        }
    }

    pub(super) fn parse_block(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        self.expect(&TokenKind::Punct(Punct::LBrace))?;

        let mut declarations = Vec::new();

        while !self.check(&TokenKind::Punct(Punct::RBrace)) && !self.is_at_end() {
            declarations.push(self.parse_declaration()?);
        }

        let close = self.expect(&TokenKind::Punct(Punct::RBrace))?;

        Ok(Statement {
            kind: StatementKind::Block(declarations),
            span: Span::merge(start, close.span),
        })
    }

    pub(crate) fn parse_call_args(&mut self) -> Result<Vec<Expression>, ParserError> {
        let mut args = Vec::new();

        if self.check(&TokenKind::Punct(Punct::RParen)) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if self.check(&TokenKind::Punct(Punct::Comma)) {
                self.advance();
                continue;
            }

            break;
        }

        Ok(args)
    }

    pub(crate) fn parse_postfix(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::Punct(Punct::LParen)) {
                let name = match &expression.kind {
                    ExpressionKind::Identifier(s) => s.clone(),
                    _ => {
                        return Err(ParserError {
                            message: "can only call functions by name".to_string(),
                            span: expression.span,
                        });
                    }
                };

                let call_start = expression.span;
                let open_span = self.peek().span;
                self.advance();
                let args = self.parse_call_args()?;
                let close = self.expect_closing(
                    &TokenKind::Punct(Punct::RParen),
                    &TokenKind::Punct(Punct::LParen),
                    open_span,
                )?;
                expression = Expression {
                    kind: ExpressionKind::Call { callee: name, args },
                    span: Span::merge(call_start, close.span),
                };
            } else if self.check(&TokenKind::Punct(Punct::LBracket)) {
                let open_span = self.peek().span;
                self.advance();
                let index = self.parse_expression()?;
                let close = self.expect_closing(
                    &TokenKind::Punct(Punct::RBracket),
                    &TokenKind::Punct(Punct::LBracket),
                    open_span,
                )?;
                let target_span = expression.span;
                let close_span = close.span;
                expression = Expression {
                    kind: ExpressionKind::Index {
                        target: Box::new(expression),
                        index: Box::new(index),
                    },
                    span: Span::merge(target_span, close_span),
                };
            } else if self.check(&TokenKind::Punct(Punct::Dot)) {
                let target_span = expression.span;
                self.advance();
                let field_token = self.advance();
                let field = match field_token.kind {
                    TokenKind::Identifier(s) => s,
                    other => {
                        return Err(ParserError {
                            message: format!("expected: a field name after `.`, got: {other}"),
                            span: field_token.span,
                        });
                    }
                };
                expression = Expression {
                    kind: ExpressionKind::Member {
                        target: Box::new(expression),
                        field,
                    },
                    span: Span::merge(target_span, field_token.span),
                };
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_for_step(&mut self) -> Result<ForStep, ParserError> {
        if self.check(&TokenKind::Operator(Operator::Increment))
            || self.check(&TokenKind::Operator(Operator::Decrement))
        {
            let operator = self.advance();
            let target = self.parse_primary()?;
            let for_step = match operator.kind {
                TokenKind::Operator(Operator::Increment) => ForStep::Increment(target),
                TokenKind::Operator(Operator::Decrement) => ForStep::Decrement(target),
                _ => unreachable!(),
            };
            return Ok(for_step);
        }

        let target = self.parse_expression()?;
        self.expect(&TokenKind::Operator(Operator::Assign))?;
        let value = self.parse_expression()?;
        Ok(ForStep::Assign { target, value })
    }

    fn parse_expression_or_assign(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        let expression = self.parse_expression()?;

        if self.check(&TokenKind::Operator(Operator::Assign)) {
            self.advance();
            let value = self.parse_expression()?;
            let semi = self.expect(&TokenKind::Punct(Punct::Semicolon))?;
            Ok(Statement {
                kind: StatementKind::Assign {
                    target: expression,
                    value,
                },
                span: Span::merge(start, semi.span),
            })
        } else {
            let semi = self.expect(&TokenKind::Punct(Punct::Semicolon))?;
            Ok(Statement {
                kind: StatementKind::ExpressionStatement(expression),
                span: Span::merge(start, semi.span),
            })
        }
    }

    fn parse_if(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        self.advance();

        self.expect(&TokenKind::Punct(Punct::LParen))?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Punct(Punct::RParen))?;

        let then_branch = Box::new(self.parse_statement()?);

        let else_branch = if self.check(&TokenKind::Keyword(Keyword::Else)) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        let end_span = match else_branch {
            Some(ref b) => b.span,
            None => then_branch.span,
        };

        Ok(Statement {
            kind: StatementKind::If {
                condition,
                then_branch,
                else_branch,
            },
            span: Span::merge(start, end_span),
        })
    }

    fn parse_while(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        self.advance();

        self.expect(&TokenKind::Punct(Punct::LParen))?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Punct(Punct::RParen))?;

        let body = Box::new(self.parse_statement()?);
        let body_span = body.span;

        Ok(Statement {
            kind: StatementKind::While { condition, body },
            span: Span::merge(start, body_span),
        })
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        self.advance();

        let value = if self.check(&TokenKind::Punct(Punct::Semicolon)) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let semi = self.expect(&TokenKind::Punct(Punct::Semicolon))?;

        Ok(Statement {
            kind: StatementKind::Return(value),
            span: Span::merge(start, semi.span),
        })
    }

    fn parse_incdec(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        let op_token = self.advance();

        let target = self.parse_primary()?;
        let semi = self.expect(&TokenKind::Punct(Punct::Semicolon))?;

        let kind = match op_token.kind {
            TokenKind::Operator(Operator::Increment) => StatementKind::Increment(target),
            TokenKind::Operator(Operator::Decrement) => StatementKind::Decrement(target),
            _ => unreachable!(),
        };

        Ok(Statement {
            kind,
            span: Span::merge(start, semi.span),
        })
    }

    fn parse_for(&mut self) -> Result<Statement, ParserError> {
        let start = self.peek().span;
        self.advance();

        self.expect(&TokenKind::Punct(Punct::LParen))?;

        let init: ForInit = if self.check(&TokenKind::Punct(Punct::Semicolon)) {
            self.advance();
            ForInit::None
        } else if self.check(&TokenKind::Keyword(Keyword::Let)) {
            let declaration = self.parse_let()?;
            ForInit::Let(Box::new(declaration))
        } else {
            let step = self.parse_for_step()?;
            self.expect(&TokenKind::Punct(Punct::Semicolon))?;
            ForInit::Step(step)
        };

        let condition = if self.check(&TokenKind::Punct(Punct::Semicolon)) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.expect(&TokenKind::Punct(Punct::Semicolon))?;

        let step = if self.check(&TokenKind::Punct(Punct::RParen)) {
            None
        } else {
            Some(self.parse_for_step()?)
        };

        self.expect(&TokenKind::Punct(Punct::RParen))?;

        let body = Box::new(self.parse_statement()?);
        let body_span = body.span;

        Ok(Statement {
            kind: StatementKind::For {
                init,
                condition,
                step,
                body,
            },
            span: Span::merge(start, body_span),
        })
    }
}
