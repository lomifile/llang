use super::{num, op, parser, span};
use ast::statement::{Expression, ExpressionKind};
use token::keywords::{Keyword, Operator, Punct};
use token::token::TokenKind;

fn expr(kinds: Vec<TokenKind>) -> Expression {
    parser(kinds)
        .parse_expression()
        .expect("expression should parse")
}

fn kind(kinds: Vec<TokenKind>) -> ExpressionKind {
    expr(kinds).kind
}

fn lit(n: f64) -> Expression {
    Expression {
        kind: ExpressionKind::LiteralNumber(n),
        span: span(1, 1),
    }
}

fn bin(o: Operator, left: Expression, right: Expression) -> ExpressionKind {
    ExpressionKind::BinaryOp {
        op: o,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn unary(o: Operator, operand: Expression) -> ExpressionKind {
    ExpressionKind::UnaryOp {
        op: o,
        operand: Box::new(operand),
    }
}

fn at(mut e: Expression, col: u32) -> Expression {
    e.span = span(col, 1);
    e
}

#[test]
fn parse_primary_reads_every_literal_form() {
    assert_eq!(kind(vec![num(4.5)]), ExpressionKind::LiteralNumber(4.5));
    assert_eq!(
        kind(vec![TokenKind::Str("hi".to_string())]),
        ExpressionKind::LiteralString("hi".to_string())
    );
    assert_eq!(
        kind(vec![TokenKind::Identifier("x".to_string())]),
        ExpressionKind::Identifier("x".to_string())
    );
    assert_eq!(
        kind(vec![TokenKind::Keyword(Keyword::True)]),
        ExpressionKind::LiteralBoolean(true)
    );
    assert_eq!(
        kind(vec![TokenKind::Keyword(Keyword::False)]),
        ExpressionKind::LiteralBoolean(false)
    );
    assert_eq!(
        kind(vec![TokenKind::Keyword(Keyword::Null)]),
        ExpressionKind::LiteralNull
    );
}

#[test]
fn parse_primary_carries_the_token_span() {
    let mut p = parser(vec![op(Operator::Minus), num(1.0)]);
    p.advance();

    let parsed = p.parse_primary().expect("number should parse");

    assert_eq!(parsed.span, span(2, 1));
}

#[test]
fn parse_primary_rejects_a_non_expression_token() {
    let error = parser(vec![TokenKind::Punct(Punct::RBrace)])
        .parse_primary()
        .expect_err("a closing brace starts no expression");

    assert_eq!(
        error.message,
        "expected: an expression, got: punctuation `}`"
    );
    assert_eq!(error.span, span(1, 1));
}

#[test]
fn parse_unary_applies_prefix_operators_right_to_left() {
    assert_eq!(
        kind(vec![op(Operator::Minus), num(1.0)]),
        unary(Operator::Minus, at(lit(1.0), 2))
    );
    assert_eq!(
        kind(vec![op(Operator::Not), op(Operator::Not), num(1.0)]),
        unary(
            Operator::Not,
            Expression {
                kind: unary(Operator::Not, at(lit(1.0), 3)),
                span: span(2, 2),
            }
        )
    );
}

#[test]
fn parse_unary_binds_tighter_than_factor() {
    assert_eq!(
        kind(vec![
            op(Operator::Minus),
            num(2.0),
            op(Operator::Multiply),
            num(3.0)
        ]),
        bin(
            Operator::Multiply,
            Expression {
                kind: unary(Operator::Minus, at(lit(2.0), 2)),
                span: span(1, 2),
            },
            at(lit(3.0), 4)
        )
    );
}

#[test]
fn parse_factor_is_left_associative() {
    assert_eq!(
        kind(vec![
            num(8.0),
            op(Operator::Divide),
            num(4.0),
            op(Operator::Modulo),
            num(3.0)
        ]),
        bin(
            Operator::Modulo,
            Expression {
                kind: bin(Operator::Divide, lit(8.0), at(lit(4.0), 3)),
                span: span(1, 3),
            },
            at(lit(3.0), 5)
        )
    );
}

#[test]
fn parse_term_is_left_associative() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            op(Operator::Minus),
            num(3.0)
        ]),
        bin(
            Operator::Minus,
            Expression {
                kind: bin(Operator::Plus, lit(1.0), at(lit(2.0), 3)),
                span: span(1, 3),
            },
            at(lit(3.0), 5)
        )
    );
}

#[test]
fn parse_term_gives_factor_higher_precedence() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            op(Operator::Multiply),
            num(3.0)
        ]),
        bin(
            Operator::Plus,
            lit(1.0),
            Expression {
                kind: bin(Operator::Multiply, at(lit(2.0), 3), at(lit(3.0), 5)),
                span: span(3, 3),
            }
        )
    );
}

#[test]
fn parse_comparison_sits_below_term() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            op(Operator::LessThan),
            num(4.0)
        ]),
        bin(
            Operator::LessThan,
            Expression {
                kind: bin(Operator::Plus, lit(1.0), at(lit(2.0), 3)),
                span: span(1, 3),
            },
            at(lit(4.0), 5)
        )
    );
}

#[test]
fn parse_comparison_accepts_each_relational_operator() {
    for o in [
        Operator::GreaterThan,
        Operator::LessThan,
        Operator::GreaterOrEqual,
        Operator::LessOrEqual,
    ] {
        assert_eq!(
            kind(vec![num(1.0), op(o), num(2.0)]),
            bin(o, lit(1.0), at(lit(2.0), 3)),
            "{o} should parse as a comparison"
        );
    }
}

#[test]
fn parse_equality_sits_below_comparison() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::LessThan),
            num(2.0),
            op(Operator::Eq),
            TokenKind::Keyword(Keyword::True)
        ]),
        bin(
            Operator::Eq,
            Expression {
                kind: bin(Operator::LessThan, lit(1.0), at(lit(2.0), 3)),
                span: span(1, 3),
            },
            Expression {
                kind: ExpressionKind::LiteralBoolean(true),
                span: span(5, 1),
            }
        )
    );
}

#[test]
fn parse_equality_accepts_both_operators() {
    for o in [Operator::Eq, Operator::NotEq] {
        assert_eq!(
            kind(vec![num(1.0), op(o), num(2.0)]),
            bin(o, lit(1.0), at(lit(2.0), 3)),
            "{o} should parse as an equality"
        );
    }
}

#[test]
fn parse_logic_and_sits_below_equality() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::Eq),
            num(2.0),
            op(Operator::And),
            TokenKind::Keyword(Keyword::True)
        ]),
        bin(
            Operator::And,
            Expression {
                kind: bin(Operator::Eq, lit(1.0), at(lit(2.0), 3)),
                span: span(1, 3),
            },
            Expression {
                kind: ExpressionKind::LiteralBoolean(true),
                span: span(5, 1),
            }
        )
    );
}

#[test]
fn parse_logic_or_sits_below_logic_and() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::Or),
            num(2.0),
            op(Operator::And),
            num(3.0)
        ]),
        bin(
            Operator::Or,
            lit(1.0),
            Expression {
                kind: bin(Operator::And, at(lit(2.0), 3), at(lit(3.0), 5)),
                span: span(3, 3),
            }
        )
    );
}

#[test]
fn parse_logic_or_is_left_associative() {
    assert_eq!(
        kind(vec![
            num(1.0),
            op(Operator::Or),
            num(2.0),
            op(Operator::Or),
            num(3.0)
        ]),
        bin(
            Operator::Or,
            Expression {
                kind: bin(Operator::Or, lit(1.0), at(lit(2.0), 3)),
                span: span(1, 3),
            },
            at(lit(3.0), 5)
        )
    );
}

#[test]
fn parentheses_override_precedence() {
    assert_eq!(
        kind(vec![
            TokenKind::Punct(Punct::LParen),
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            TokenKind::Punct(Punct::RParen),
            op(Operator::Multiply),
            num(3.0)
        ]),
        bin(
            Operator::Multiply,
            Expression {
                kind: bin(Operator::Plus, at(lit(1.0), 2), at(lit(2.0), 4)),
                span: span(2, 3),
            },
            at(lit(3.0), 7)
        )
    );
}

#[test]
fn an_unclosed_group_reports_the_missing_paren() {
    let error = parser(vec![TokenKind::Punct(Punct::LParen), num(1.0)])
        .parse_expression()
        .expect_err("the group is never closed");

    assert_eq!(
        error.message,
        "expected: punctuation `)` to close punctuation `(` opened at 1:1, got: end of input"
    );
}

#[test]
fn a_missing_right_operand_reports_the_offending_token() {
    let error = parser(vec![num(1.0), op(Operator::Plus)])
        .parse_expression()
        .expect_err("plus has no right operand");

    assert_eq!(error.message, "expected: an expression, got: end of input");
    assert_eq!(error.span, span(3, 0));
}

#[test]
fn a_binary_span_covers_both_operands() {
    assert_eq!(
        expr(vec![
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            op(Operator::Multiply),
            num(3.0)
        ])
        .span,
        span(1, 5)
    );
}

#[test]
fn a_unary_span_covers_the_operator_and_its_operand() {
    assert_eq!(
        expr(vec![op(Operator::Minus), op(Operator::Not), num(1.0)]).span,
        span(1, 3)
    );
}

#[test]
fn a_group_span_covers_only_the_inner_expression() {
    assert_eq!(
        expr(vec![
            TokenKind::Punct(Punct::LParen),
            num(1.0),
            op(Operator::Plus),
            num(2.0),
            TokenKind::Punct(Punct::RParen)
        ])
        .span,
        span(2, 3)
    );
}

#[test]
fn parse_expression_stops_at_the_first_token_it_cannot_use() {
    let mut p = parser(vec![
        num(1.0),
        op(Operator::Plus),
        num(2.0),
        TokenKind::Punct(Punct::Semicolon),
    ]);

    p.parse_expression().expect("the sum should parse");

    assert_eq!(p.peek().kind, TokenKind::Punct(Punct::Semicolon));
}

#[test]
fn an_empty_array_literal_parses() {
    assert_eq!(
        kind(vec![
            TokenKind::Punct(Punct::LBracket),
            TokenKind::Punct(Punct::RBracket),
        ]),
        ExpressionKind::ArrayLiteral(vec![])
    );
}

#[test]
fn an_array_literal_parses_its_elements() {
    assert_eq!(
        kind(vec![
            TokenKind::Punct(Punct::LBracket),
            num(1.0),
            TokenKind::Punct(Punct::Comma),
            num(2.0),
            TokenKind::Punct(Punct::RBracket),
        ]),
        ExpressionKind::ArrayLiteral(vec![at(lit(1.0), 2), at(lit(2.0), 4)])
    );
}

#[test]
fn an_array_span_covers_both_brackets() {
    assert_eq!(
        expr(vec![
            TokenKind::Punct(Punct::LBracket),
            num(1.0),
            TokenKind::Punct(Punct::RBracket),
        ])
        .span,
        span(1, 3)
    );
}

#[test]
fn an_unclosed_array_points_back_at_its_opening_bracket() {
    let error = parser(vec![TokenKind::Punct(Punct::LBracket), num(1.0)])
        .parse_expression()
        .expect_err("the array is never closed");

    assert_eq!(
        error.message,
        "expected: punctuation `]` to close punctuation `[` opened at 1:1, got: end of input"
    );
}

#[test]
fn an_unclosed_object_points_back_at_its_opening_brace() {
    let error = parser(vec![
        TokenKind::Punct(Punct::LBrace),
        TokenKind::Identifier("a".to_string()),
        TokenKind::Punct(Punct::Colon),
        num(1.0),
    ])
    .parse_expression()
    .expect_err("the object is never closed");

    assert_eq!(
        error.message,
        "expected: punctuation `}` to close punctuation `{` opened at 1:1, got: end of input"
    );
}

#[test]
fn an_unclosed_delimiter_is_reported_at_the_token_that_should_have_closed_it() {
    let error = parser(vec![TokenKind::Punct(Punct::LBracket), num(1.0)])
        .parse_expression()
        .expect_err("the array is never closed");

    assert_eq!(error.span, span(3, 0));
}

#[test]
fn an_empty_object_literal_parses() {
    assert_eq!(
        kind(vec![
            TokenKind::Punct(Punct::LBrace),
            TokenKind::Punct(Punct::RBrace),
        ]),
        ExpressionKind::ObjectLiteral(vec![])
    );
}

#[test]
fn an_object_literal_takes_identifier_and_string_keys() {
    assert_eq!(
        kind(vec![
            TokenKind::Punct(Punct::LBrace),
            TokenKind::Identifier("a".to_string()),
            TokenKind::Punct(Punct::Colon),
            num(1.0),
            TokenKind::Punct(Punct::Comma),
            TokenKind::Str("b".to_string()),
            TokenKind::Punct(Punct::Colon),
            num(2.0),
            TokenKind::Punct(Punct::RBrace),
        ]),
        ExpressionKind::ObjectLiteral(vec![
            ("a".to_string(), at(lit(1.0), 4)),
            ("b".to_string(), at(lit(2.0), 8)),
        ])
    );
}

#[test]
fn an_object_key_must_be_an_identifier_or_a_string() {
    let error = parser(vec![
        TokenKind::Punct(Punct::LBrace),
        num(1.0),
        TokenKind::Punct(Punct::Colon),
        num(2.0),
        TokenKind::Punct(Punct::RBrace),
    ])
    .parse_expression()
    .expect_err("a number is not a key");

    assert_eq!(error.message, "expected: an object key, got: number `1`");
}

#[test]
fn a_call_with_no_arguments_parses() {
    assert_eq!(
        kind(vec![
            TokenKind::Identifier("f".to_string()),
            TokenKind::Punct(Punct::LParen),
            TokenKind::Punct(Punct::RParen),
        ]),
        ExpressionKind::Call {
            callee: "f".to_string(),
            args: vec![],
        }
    );
}

#[test]
fn a_call_parses_its_arguments() {
    assert_eq!(
        kind(vec![
            TokenKind::Identifier("f".to_string()),
            TokenKind::Punct(Punct::LParen),
            num(1.0),
            TokenKind::Punct(Punct::Comma),
            num(2.0),
            TokenKind::Punct(Punct::RParen),
        ]),
        ExpressionKind::Call {
            callee: "f".to_string(),
            args: vec![at(lit(1.0), 3), at(lit(2.0), 5)],
        }
    );
}

#[test]
fn a_call_span_covers_the_callee_and_the_closing_paren() {
    assert_eq!(
        expr(vec![
            TokenKind::Identifier("f".to_string()),
            TokenKind::Punct(Punct::LParen),
            TokenKind::Punct(Punct::RParen),
        ])
        .span,
        span(1, 3)
    );
}

#[test]
fn only_a_named_callee_can_be_called() {
    let error = parser(vec![
        num(1.0),
        TokenKind::Punct(Punct::LParen),
        TokenKind::Punct(Punct::RParen),
    ])
    .parse_expression()
    .expect_err("a number is not callable");

    assert_eq!(error.message, "can only call functions by name");
}

#[test]
fn member_access_parses() {
    assert_eq!(
        kind(vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Punct(Punct::Dot),
            TokenKind::Identifier("b".to_string()),
        ]),
        ExpressionKind::Member {
            target: Box::new(Expression {
                kind: ExpressionKind::Identifier("a".to_string()),
                span: span(1, 1),
            }),
            field: "b".to_string(),
        }
    );
}

#[test]
fn a_member_field_must_be_an_identifier() {
    let error = parser(vec![
        TokenKind::Identifier("a".to_string()),
        TokenKind::Punct(Punct::Dot),
        num(1.0),
    ])
    .parse_expression()
    .expect_err("a number is not a field name");

    assert_eq!(
        error.message,
        "expected: a field name after `.`, got: number `1`"
    );
}

#[test]
fn indexing_parses() {
    assert_eq!(
        kind(vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Punct(Punct::LBracket),
            num(0.0),
            TokenKind::Punct(Punct::RBracket),
        ]),
        ExpressionKind::Index {
            target: Box::new(Expression {
                kind: ExpressionKind::Identifier("a".to_string()),
                span: span(1, 1),
            }),
            index: Box::new(at(lit(0.0), 3)),
        }
    );
}

#[test]
fn an_index_span_covers_the_target_through_the_closing_bracket() {
    assert_eq!(
        expr(vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Punct(Punct::LBracket),
            num(0.0),
            TokenKind::Punct(Punct::RBracket),
        ])
        .span,
        span(1, 4)
    );
}

#[test]
fn postfix_operators_chain_left_to_right() {
    assert_eq!(
        kind(vec![
            TokenKind::Identifier("f".to_string()),
            TokenKind::Punct(Punct::LParen),
            TokenKind::Punct(Punct::RParen),
            TokenKind::Punct(Punct::Dot),
            TokenKind::Identifier("b".to_string()),
        ]),
        ExpressionKind::Member {
            target: Box::new(Expression {
                kind: ExpressionKind::Call {
                    callee: "f".to_string(),
                    args: vec![],
                },
                span: span(1, 3),
            }),
            field: "b".to_string(),
        }
    );
}

#[test]
fn a_unary_operator_applies_to_the_whole_postfix_chain() {
    assert_eq!(
        kind(vec![
            op(Operator::Minus),
            TokenKind::Identifier("f".to_string()),
            TokenKind::Punct(Punct::LParen),
            TokenKind::Punct(Punct::RParen),
        ]),
        unary(
            Operator::Minus,
            Expression {
                kind: ExpressionKind::Call {
                    callee: "f".to_string(),
                    args: vec![],
                },
                span: span(2, 3),
            }
        )
    );
}
