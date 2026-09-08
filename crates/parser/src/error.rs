use token::token::Span;

pub struct ParserError {
    pub message: String,
    pub span: Span,
}
