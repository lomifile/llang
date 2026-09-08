#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Let,
    Const,
    Function,
    If,
    Else,
    For,
    While,
    Return,
    True,
    False,
    Null,
}

impl Keyword {
    const ALL: &'static [(Self, &'static str)] = &[
        (Self::Let, "let"),
        (Self::Const, "const"),
        (Self::Function, "function"),
        (Self::If, "if"),
        (Self::Else, "else"),
        (Self::For, "for"),
        (Self::While, "while"),
        (Self::Return, "return"),
        (Self::True, "True"),
        (Self::False, "False"),
        (Self::Null, "Null"),
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Let => "let",
            Self::Const => "const",
            Self::Function => "function",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::While => "while",
            Self::Return => "return",
            Self::True => "True",
            Self::False => "False",
            Self::Null => "Null",
        }
    }

    pub fn lookup(s: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .find_map(|&(kw, text)| (text == s).then_some(kw))
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Eq,
    NotEq,
    Increment,
    Decrement,
    And,
    Or,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Plus,
    Minus,
    Not,
}

impl Operator {
    const ALL: &'static [(Self, &'static str)] = &[
        (Self::Eq, "==="),
        (Self::NotEq, "!=="),
        (Self::GreaterOrEqual, ">="),
        (Self::LessOrEqual, "<="),
        (Self::Increment, "++"),
        (Self::Decrement, "--"),
        (Self::And, "&&"),
        (Self::Or, "||"),
        (Self::GreaterThan, ">"),
        (Self::LessThan, "<"),
        (Self::Multiply, "*"),
        (Self::Divide, "/"),
        (Self::Modulo, "%"),
        (Self::Plus, "+"),
        (Self::Assign, "="),
        (Self::Minus, "-"),
        (Self::Not, "!"),
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "===",
            Self::NotEq => "!==",
            Self::Increment => "++",
            Self::Decrement => "--",
            Self::And => "&&",
            Self::Or => "||",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::Assign => "=",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
        }
    }

    pub fn strip_prefix(src: &[char]) -> Option<(Self, usize)> {
        let mut candidates: [&(Self, &'static str); Self::ALL.len()] =
            std::array::from_fn(|i| &Self::ALL[i]);
        candidates.sort_by_key(|b| std::cmp::Reverse(b.1.len()));
        candidates
            .iter()
            .find(|(_, text)| {
                let op_chars: Vec<char> = text.chars().collect();
                src.len() >= op_chars.len() && src[..op_chars.len()] == op_chars[..]
            })
            .map(|&&(op, text)| (op, text.chars().count()))
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punct {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Semicolon,
    Comma,
    Dot,
}

impl Punct {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::Colon => ":",
            Self::Semicolon => ";",
            Self::Comma => ",",
            Self::Dot => ".",
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            '(' => Self::LParen,
            ')' => Self::RParen,
            '{' => Self::LBrace,
            '}' => Self::RBrace,
            '[' => Self::LBracket,
            ']' => Self::RBracket,
            ':' => Self::Colon,
            ';' => Self::Semicolon,
            ',' => Self::Comma,
            '.' => Self::Dot,
            _ => return None,
        })
    }
}

impl std::fmt::Display for Punct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
