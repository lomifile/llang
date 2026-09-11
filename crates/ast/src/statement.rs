use token::keywords::Operator;
use token::token::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Object,
    Void,
    Null,
    Array(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    LiteralNumber(f64),
    LiteralString(String),
    LiteralBoolean(bool),
    LiteralNull,
    Identifier(String),
    BinaryOp {
        op: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: Operator,
        operand: Box<Expression>,
    },
    Call {
        callee: String,
        args: Vec<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    ObjectLiteral(Vec<(String, Expression)>),
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
    },
    Member {
        target: Box<Expression>,
        field: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    ExpressionStatement(Expression),
    Assign {
        target: Expression,
        value: Expression,
    },
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        init: ForInit,
        condition: Option<Expression>,
        step: Option<ForStep>,
        body: Box<Statement>,
    },
    Return(Option<Expression>),
    Block(Vec<Declaration>),
    Increment(Expression),
    Decrement(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForStep {
    Assign {
        target: Expression,
        value: Expression,
    },
    Increment(Expression),
    Decrement(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Let(Box<Declaration>),
    Step(ForStep),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationKind {
    Let {
        name: String,
        ty: Type,
        init: Expression,
    },
    Const {
        name: String,
        ty: Type,
        init: Expression,
    },
    Function {
        name: String,
        params: Vec<FunctionParam>,
        return_type: Type,
        body: Box<Statement>,
    },
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub span: Span,
}
