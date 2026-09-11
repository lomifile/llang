use super::Parser;
use crate::error::ParserError;
use ast::statement::{Declaration, ForInit, ForStep, Statement, StatementKind};
use token::{
    keywords::{Keyword, Operator, Punct},
    token::{Span, Token, TokenKind},
};

impl Parser {
    pub(super) fn parse_program(&mut self) -> Result<Vec<Declaration>, ParserError> {
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
