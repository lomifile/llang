use super::Parser;
use crate::error::ParserError;
use ast::statement::Type;
use token::{
    keywords::{Keyword, Punct},
    token::TokenKind,
};

impl Parser {
    pub(super) fn parse_type(&mut self) -> Result<Type, ParserError> {
        let token = self.advance();
        let span = token.span;

        match token.kind {
            TokenKind::Keyword(Keyword::Null) => Ok(Type::Null),
            TokenKind::Identifier(name) => match name.as_str() {
                "Number" => Ok(Type::Number),
                "String" => Ok(Type::String),
                "Boolean" => Ok(Type::Boolean),
                "Object" => Ok(Type::Object),
                "Void" => Ok(Type::Void),
                "Array" => {
                    self.expect(&TokenKind::Punct(Punct::LBracket))?;
                    let inner = self.parse_type()?;
                    self.expect(&TokenKind::Punct(Punct::RBracket))?;
                    Ok(Type::Array(Box::new(inner)))
                }
                _ => Err(ParserError {
                    span,
                    message: format!("unknown type: {name}"),
                }),
            },
            _ => Err(ParserError {
                span,
                message: "expected a type".to_string(),
            }),
        }
    }
}
