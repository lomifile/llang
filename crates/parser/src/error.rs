use token::token::Span;

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
}
