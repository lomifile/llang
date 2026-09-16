use std::fmt;

use lexer::error::LexerError;
use parser::error::ParserError;
use types::error::TypeError;
use vm::types::RuntimeError;

#[derive(Debug)]
pub(crate) enum RunError {
    Lex(LexerError),
    Parse(ParserError),
    Type(TypeError),
    Runtime(RuntimeError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Lex(error) => write!(f, "lex error: {error}"),
            RunError::Parse(error) => write!(f, "parse error: {error}"),
            RunError::Type(error) => write!(f, "type error: {error}"),
            RunError::Runtime(error) => write!(f, "runtime error: {error}"),
        }
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RunError::Lex(error) => Some(error),
            RunError::Parse(error) => Some(error),
            RunError::Type(error) => Some(error),
            RunError::Runtime(error) => Some(error),
        }
    }
}

impl From<LexerError> for RunError {
    fn from(error: LexerError) -> Self {
        RunError::Lex(error)
    }
}

impl From<ParserError> for RunError {
    fn from(error: ParserError) -> Self {
        RunError::Parse(error)
    }
}

impl From<TypeError> for RunError {
    fn from(error: TypeError) -> Self {
        RunError::Type(error)
    }
}

impl From<RuntimeError> for RunError {
    fn from(error: RuntimeError) -> Self {
        RunError::Runtime(error)
    }
}

#[cfg(test)]
mod tests {
    use super::RunError;
    use lexer::error::LexerError;
    use parser::error::ParserError;
    use token::token::Span;
    use types::error::TypeError;
    use vm::types::RuntimeError;

    fn span(line: u32, col: u32) -> Span {
        Span { line, col, len: 1 }
    }

    #[test]
    fn a_lex_error_displays_with_its_phase() {
        let error = RunError::Lex(LexerError::UnexpectedChar {
            line: 2,
            col: 7,
            ch: '@',
        });

        assert_eq!(
            error.to_string(),
            "lex error: 2:7: unexpected character `@`: no token can start with it"
        );
    }

    #[test]
    fn a_parse_error_displays_with_its_phase() {
        let error = RunError::Parse(ParserError {
            message: "expected `;`".to_string(),
            span: span(3, 11),
        });

        assert_eq!(error.to_string(), "parse error: 3:11: expected `;`");
    }

    #[test]
    fn a_type_error_displays_with_its_phase() {
        let error = RunError::Type(TypeError {
            message: "expected Number, found Bool".to_string(),
            span: span(4, 2),
        });

        assert_eq!(
            error.to_string(),
            "type error: 4:2: expected Number, found Bool"
        );
    }

    #[test]
    fn a_runtime_error_displays_with_its_phase() {
        let error = RunError::Runtime(RuntimeError {
            message: "division by zero".to_string(),
            line: 9,
        });

        assert_eq!(error.to_string(), "runtime error: 9: division by zero");
    }

    #[test]
    fn every_variant_reports_the_underlying_error_as_its_source() {
        let error: RunError = RuntimeError {
            message: "division by zero".to_string(),
            line: 9,
        }
        .into();

        let source = std::error::Error::source(&error).expect("expected a source error");
        assert_eq!(source.to_string(), "9: division by zero");
    }
}
