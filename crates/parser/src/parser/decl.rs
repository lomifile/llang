use super::Parser;
use crate::error::ParserError;
use ast::statement::{Declaration, DeclarationKind, Expression, FunctionParam, Type};
use token::{
    keywords::{Keyword, Operator, Punct},
    token::{Span, TokenKind},
};

impl Parser {
    pub(super) fn parse_declaration(&mut self) -> Result<Declaration, ParserError> {
        match self.peek().kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_let(),
            TokenKind::Keyword(Keyword::Const) => self.parse_const(),
            TokenKind::Keyword(Keyword::Function) => self.parse_function(),
            _ => {
                let parsed = self.parse_statement()?;
                let parsed_span = parsed.span;
                Ok(Declaration {
                    kind: DeclarationKind::Statement(parsed),
                    span: parsed_span,
                })
            }
        }
    }

    pub(super) fn parse_let(&mut self) -> Result<Declaration, ParserError> {
        self.parse_variable(|name, ty, init| DeclarationKind::Let { name, ty, init })
    }

    fn parse_const(&mut self) -> Result<Declaration, ParserError> {
        self.parse_variable(|name, ty, init| DeclarationKind::Const { name, ty, init })
    }

    fn parse_variable(
        &mut self,
        build: fn(String, Type, Expression) -> DeclarationKind,
    ) -> Result<Declaration, ParserError> {
        let start = self.peek().span;
        self.advance();

        let name = self.expect_identifier("expected variable name")?;

        self.expect(&TokenKind::Punct(Punct::Colon))?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Operator(Operator::Assign))?;
        let init = self.parse_expression()?;
        let semi = self.expect(&TokenKind::Punct(Punct::Semicolon))?;

        Ok(Declaration {
            kind: build(name, ty, init),
            span: Span::merge(start, semi.span),
        })
    }

    fn parse_function(&mut self) -> Result<Declaration, ParserError> {
        let start = self.peek().span;
        self.advance();

        let name = self.expect_identifier("expected variable name")?;

        self.expect(&TokenKind::Punct(Punct::LParen))?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::Punct(Punct::RParen))?;

        self.expect(&TokenKind::Punct(Punct::Colon))?;

        let return_type = self.parse_type()?;

        let body = Box::new(self.parse_block()?);
        let body_span = body.span;

        Ok(Declaration {
            kind: DeclarationKind::Function {
                name,
                params,
                return_type,
                body,
            },
            span: Span::merge(start, body_span),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<FunctionParam>, ParserError> {
        let mut params = Vec::new();

        if self.check(&TokenKind::Punct(Punct::RParen)) {
            return Ok(params);
        }

        loop {
            let name = self.expect_identifier("expected param name")?;

            self.expect(&TokenKind::Punct(Punct::Colon))?;
            let param_type = self.parse_type()?;
            params.push(FunctionParam { name, param_type });

            if self.check(&TokenKind::Punct(Punct::Comma)) {
                self.advance();
                continue;
            }
            break;
        }

        Ok(params)
    }
}
