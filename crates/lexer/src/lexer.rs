use crate::conversion::usize_to_u32;
use crate::error::LexerError;
use token::keywords::{Keyword, Operator, Punct};
use token::token::{Span, Token, TokenKind};

#[derive(Debug)]
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    // Use mut to copy value
    // increment pos + 1 or col if '\n'
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn read_identifier(&mut self, start_line: u32, start_col: u32) -> Token {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match Keyword::lookup(&text) {
            Some(kw) => TokenKind::Keyword(kw),
            None => TokenKind::Identifier(text.clone()),
        };

        Token::new(
            kind,
            Span {
                line: start_line,
                col: start_col,
                len: usize_to_u32(text.chars().count()),
            },
        )
    }

    fn read_number(&mut self, start_line: u32, start_col: u32) -> Result<Token, LexerError> {
        let mut text = String::new();
        let mut seen_dot = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else if c == '.' && !seen_dot {
                seen_dot = true;
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match text.parse::<f64>() {
            Ok(value) => Ok(Token::new(
                TokenKind::Number(value),
                Span {
                    line: start_line,
                    col: start_col,
                    len: usize_to_u32(text.chars().count()),
                },
            )),
            Err(_) => Err(LexerError::InvalidNumber {
                line: start_line,
                col: start_col,
                text,
            }),
        }
    }

    fn read_string(&mut self, start_line: u32, start_col: u32) -> Result<Token, LexerError> {
        self.advance();
        let mut text = String::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    let len = usize_to_u32(text.chars().count());

                    return Ok(Token::new(
                        TokenKind::Str(text),
                        Span {
                            line: start_line,
                            col: start_col,
                            len,
                        },
                    ));
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n') => text.push('\n'),
                        Some('t') => text.push('\t'),
                        Some('\\') => text.push('\\'),
                        Some('"') => text.push('"'),
                        Some(other) => text.push(other),
                        None => {
                            return Err(LexerError::UnterminatedString {
                                line: start_line,
                                col: start_col,
                            });
                        }
                    }
                }
                Some(c) => {
                    text.push(self.advance().ok_or(LexerError::UnexpectedChar {
                        line: start_line,
                        col: start_col,
                        ch: c,
                    })?);
                }
                None => {
                    return Err(LexerError::UnterminatedString {
                        line: start_line,
                        col: start_col,
                    });
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
                continue;
            }

            if c == '/' && self.peek_next() == Some('/') {
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            let start_line = self.line;
            let start_col = self.col;

            if c.is_alphabetic() || c == '_' {
                let token = self.read_identifier(start_line, start_col);
                tokens.push(token);
                continue;
            }

            if c.is_ascii_digit() {
                let token = self.read_number(start_line, start_col)?;
                tokens.push(token);
                continue;
            }

            if c == '"' {
                let token = self.read_string(start_line, start_col)?;
                tokens.push(token);
                continue;
            }

            if let Some(punct) = Punct::from_char(c) {
                self.advance();
                tokens.push(Token::new(
                    TokenKind::Punct(punct),
                    Span {
                        line: start_line,
                        col: start_col,
                        len: 1,
                    },
                ));
                continue;
            }

            if let Some((op, len)) = Operator::strip_prefix(&self.chars[self.pos..]) {
                for _ in 0..len {
                    self.advance();
                }
                tokens.push(Token::new(
                    TokenKind::Operator(op),
                    Span {
                        line: start_line,
                        col: start_col,
                        len: usize_to_u32(len),
                    },
                ));
                continue;
            }

            return Err(LexerError::UnexpectedChar {
                line: start_line,
                col: start_col,
                ch: c,
            });
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            Span {
                line: self.line,
                col: self.col,
                len: 0,
            },
        ));
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::LexerError;
    use crate::lexer::Lexer;
    use token::token::{Token, TokenKind};

    #[test]
    fn test_tokenize_string_line() {
        let line = "let test: String = \"testing\"";
        let mut lexer = Lexer::new(line);
        let result = lexer.tokenize();

        match result {
            Ok(tokenized) => {
                println!("Tokenization successful:");
                for token in &tokenized {
                    println!("  {token}");
                }
            }
            Err(err) => {
                println!("tokenization err: {err}");
            }
        }
    }

    #[test]
    fn test_tokenize_int_line() {
        let line = "let test: Number = 20";
        let mut lexer = Lexer::new(line);
        let result = lexer.tokenize();

        match result {
            Ok(tokenized) => {
                println!("Tokenization successful:");
                for token in &tokenized {
                    println!("  {token}");
                }
            }
            Err(err) => {
                println!("tokenization err: {err}");
            }
        }
    }

    #[test]
    fn test_tokenize_float_line() {
        let line = "let test: Number = 20.21";
        let mut lexer = Lexer::new(line);
        let result = lexer.tokenize();

        match result {
            Ok(tokenized) => {
                println!("Tokenization successful:");
                for token in &tokenized {
                    println!("  {token}");
                }
            }
            Err(err) => {
                println!("tokenization err: {err}");
            }
        }
    }

    fn tokens(src: &str) -> Vec<Token> {
        Lexer::new(src).tokenize().expect("expected a token stream")
    }

    fn kinds(src: &str) -> Vec<String> {
        tokens(src).iter().map(|t| t.kind.to_string()).collect()
    }

    fn err(src: &str) -> LexerError {
        Lexer::new(src)
            .tokenize()
            .expect_err("expected a lexer error")
    }

    fn numbers(src: &str) -> Vec<f64> {
        tokens(src)
            .into_iter()
            .filter_map(|t| match t.kind {
                TokenKind::Number(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_eq_is_one_operator() {
        assert_eq!(
            kinds("a === b"),
            [
                "identifier `a`",
                "operator `===`",
                "identifier `b`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_not_eq_is_one_operator() {
        assert_eq!(
            kinds("a !== b"),
            [
                "identifier `a`",
                "operator `!==`",
                "identifier `b`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_relational_operators_beat_single_char() {
        assert_eq!(
            kinds("a >= b"),
            [
                "identifier `a`",
                "operator `>=`",
                "identifier `b`",
                "end of input"
            ]
        );
        assert_eq!(
            kinds("a <= b"),
            [
                "identifier `a`",
                "operator `<=`",
                "identifier `b`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_increment_and_decrement() {
        assert_eq!(
            kinds("i++"),
            ["identifier `i`", "operator `++`", "end of input"]
        );
        assert_eq!(
            kinds("i--"),
            ["identifier `i`", "operator `--`", "end of input"]
        );
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(
            kinds("a && b"),
            [
                "identifier `a`",
                "operator `&&`",
                "identifier `b`",
                "end of input"
            ]
        );
        assert_eq!(
            kinds("a || b"),
            [
                "identifier `a`",
                "operator `||`",
                "identifier `b`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_assign_stays_single() {
        assert_eq!(
            kinds("a = b"),
            [
                "identifier `a`",
                "operator `=`",
                "identifier `b`",
                "end of input"
            ]
        );
        assert_eq!(
            kinds("a === b"),
            [
                "identifier `a`",
                "operator `===`",
                "identifier `b`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_operator_without_whitespace() {
        assert_eq!(
            kinds("1+2"),
            ["number `1`", "operator `+`", "number `2`", "end of input"]
        );
    }

    #[test]
    fn test_integer_value() {
        assert_eq!(numbers("42"), [42.0]);
        assert_eq!(kinds("42"), ["number `42`", "end of input"]);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_float_value() {
        assert_eq!(numbers("3.14"), [3.14]);
    }

    #[test]
    fn test_leading_zero_float() {
        assert_eq!(numbers("0.5"), [0.5]);
        assert_eq!(kinds("0.5"), ["number `0.5`", "end of input"]);
    }

    #[test]
    fn test_separate_numbers() {
        assert_eq!(numbers("1 2 3"), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_double_dot_number_splits_on_second_dot() {
        assert_eq!(
            kinds("1.2.3"),
            [
                "number `1.2`",
                "punctuation `.`",
                "number `3`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_trailing_comment_is_skipped() {
        assert_eq!(
            kinds("let x = 1; // trailing"),
            [
                "keyword `let`",
                "identifier `x`",
                "operator `=`",
                "number `1`",
                "punctuation `;`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_comment_only_line() {
        assert_eq!(kinds("// comment"), ["end of input"]);
    }

    #[test]
    fn test_single_slash_is_divide() {
        assert_eq!(
            kinds("a / b"),
            [
                "identifier `a`",
                "operator `/`",
                "identifier `b`",
                "end of input"
            ]
        );
    }

    #[test]
    fn test_code_after_comment_line() {
        let toks = tokens("// header\nlet a = 1;");
        assert_eq!(
            toks.iter().map(|t| t.kind.to_string()).collect::<Vec<_>>(),
            [
                "keyword `let`",
                "identifier `a`",
                "operator `=`",
                "number `1`",
                "punctuation `;`",
                "end of input"
            ]
        );
        assert_eq!((toks[0].span.line, toks[0].span.col), (2, 1));
    }

    #[test]
    fn test_second_line_span() {
        let toks = tokens("let a = 1;\nlet b = 2;");
        let second_let = &toks[5];
        assert_eq!(second_let.kind.to_string(), "keyword `let`");
        assert_eq!((second_let.span.line, second_let.span.col), (2, 1));
        assert_eq!((toks[0].span.line, toks[0].span.col), (1, 1));
    }

    #[test]
    fn test_span_after_blank_line() {
        let toks = tokens("a\n\nb");
        assert_eq!((toks[0].span.line, toks[0].span.col), (1, 1));
        assert_eq!((toks[1].span.line, toks[1].span.col), (3, 1));
    }

    #[test]
    fn test_unterminated_string() {
        assert_eq!(
            err("\"unterminated"),
            LexerError::UnterminatedString { line: 1, col: 1 }
        );
    }

    #[test]
    fn test_newline_does_not_close_string() {
        assert_eq!(
            err("let a = \"line one\n"),
            LexerError::UnterminatedString { line: 1, col: 9 }
        );
    }

    #[test]
    fn test_unexpected_char() {
        assert_eq!(
            err("let a = @"),
            LexerError::UnexpectedChar {
                line: 1,
                col: 9,
                ch: '@'
            }
        );
        assert_eq!(
            err("#"),
            LexerError::UnexpectedChar {
                line: 1,
                col: 1,
                ch: '#'
            }
        );
        assert_eq!(
            err("a & b"),
            LexerError::UnexpectedChar {
                line: 1,
                col: 3,
                ch: '&'
            }
        );
        assert_eq!(
            err("a | b"),
            LexerError::UnexpectedChar {
                line: 1,
                col: 3,
                ch: '|'
            }
        );
    }

    #[test]
    fn test_empty_input() {
        let toks = tokens("");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, TokenKind::Eof);
        assert_eq!((toks[0].span.line, toks[0].span.col), (1, 1));
    }

    #[test]
    fn test_full_program() {
        let src = "let name: String = \"lang\";\n\
                   const limit: Number = 10;\n\
                   // the loop below counts\n\
                   if (limit >= 10) { name = \"big\"; }\n\
                   while (limit !== 0) { --limit; }\n\
                   for (let i: Number = 1; i <= 2; ++i) {}\n";

        assert_eq!(
            kinds(src),
            [
                "keyword `let`",
                "identifier `name`",
                "punctuation `:`",
                "identifier `String`",
                "operator `=`",
                "string `\"lang\"`",
                "punctuation `;`",
                "keyword `const`",
                "identifier `limit`",
                "punctuation `:`",
                "identifier `Number`",
                "operator `=`",
                "number `10`",
                "punctuation `;`",
                "keyword `if`",
                "punctuation `(`",
                "identifier `limit`",
                "operator `>=`",
                "number `10`",
                "punctuation `)`",
                "punctuation `{`",
                "identifier `name`",
                "operator `=`",
                "string `\"big\"`",
                "punctuation `;`",
                "punctuation `}`",
                "keyword `while`",
                "punctuation `(`",
                "identifier `limit`",
                "operator `!==`",
                "number `0`",
                "punctuation `)`",
                "punctuation `{`",
                "operator `--`",
                "identifier `limit`",
                "punctuation `;`",
                "punctuation `}`",
                "keyword `for`",
                "punctuation `(`",
                "keyword `let`",
                "identifier `i`",
                "punctuation `:`",
                "identifier `Number`",
                "operator `=`",
                "number `1`",
                "punctuation `;`",
                "identifier `i`",
                "operator `<=`",
                "number `2`",
                "punctuation `;`",
                "operator `++`",
                "identifier `i`",
                "punctuation `)`",
                "punctuation `{`",
                "punctuation `}`",
                "end of input"
            ]
        );
    }
}
