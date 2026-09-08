use crate::keywords;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(keywords::Keyword),
    Operator(keywords::Operator),
    Punct(keywords::Punct),
    Identifier(String),
    Number(f64),
    Str(String),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub col: u32,
    pub len: u32,
}

impl Span {
    fn start(self) -> (u32, u32) {
        (self.line, self.col)
    }

    fn end(self) -> (u32, u32) {
        (self.line, self.col + self.len)
    }

    pub fn merge(left: Self, right: Self) -> Self {
        let start = if left.start() <= right.start() {
            left
        } else {
            right
        };
        let end = if left.end() >= right.end() {
            left
        } else {
            right
        };

        let len = if start.line == end.line {
            (end.col + end.len).saturating_sub(start.col)
        } else {
            start.len
        };

        Self {
            line: start.line,
            col: start.col,
            len,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyword(kw) => write!(f, "keyword `{kw}`"),
            Self::Operator(op) => write!(f, "operator `{op}`"),
            Self::Punct(p) => write!(f, "punctuation `{p}`"),
            Self::Identifier(name) => write!(f, "identifier `{name}`"),
            Self::Number(n) => write!(f, "number `{n}`"),
            Self::Str(s) => write!(f, "string `\"{s}\"`"),
            Self::Eof => f.write_str("end of input"),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    fn span(line: u32, col: u32, len: u32) -> Span {
        Span { line, col, len }
    }

    #[test]
    fn merge_covers_two_adjacent_spans() {
        assert_eq!(Span::merge(span(1, 1, 1), span(1, 5, 3)), span(1, 1, 7));
    }

    #[test]
    fn merge_is_order_independent() {
        let left = span(1, 1, 1);
        let right = span(1, 5, 3);

        assert_eq!(Span::merge(left, right), Span::merge(right, left));
    }

    #[test]
    fn merge_of_a_span_with_itself_is_itself() {
        let only = span(2, 4, 6);

        assert_eq!(Span::merge(only, only), only);
    }

    #[test]
    fn merge_keeps_the_wider_span_when_one_contains_the_other() {
        let outer = span(1, 1, 10);
        let inner = span(1, 3, 2);

        assert_eq!(Span::merge(outer, inner), outer);
        assert_eq!(Span::merge(inner, outer), outer);
    }

    #[test]
    fn merge_starts_at_the_earlier_line() {
        let first = span(1, 8, 1);
        let second = span(3, 2, 4);

        assert_eq!(Span::merge(first, second).line, 1);
        assert_eq!(Span::merge(first, second).col, 8);
        assert_eq!(Span::merge(second, first).line, 1);
    }
}
