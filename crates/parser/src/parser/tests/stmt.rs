use super::{at, error, statement, statement_kind};
use ast::statement::{
    Declaration, DeclarationKind, Expression, ExpressionKind, ForInit, ForStep, Statement,
    StatementKind,
};
use token::keywords::Operator;

fn ident(name: &str, col: u32, len: u32) -> Expression {
    Expression {
        kind: ExpressionKind::Identifier(name.to_string()),
        span: at(1, col, len),
    }
}

fn number(n: f64, col: u32, len: u32) -> Expression {
    Expression {
        kind: ExpressionKind::LiteralNumber(n),
        span: at(1, col, len),
    }
}

fn block_of(src: &str) -> Vec<Declaration> {
    match statement_kind(src) {
        StatementKind::Block(declarations) => declarations,
        other => panic!("`{src}` should be a block, got {other:?}"),
    }
}

fn if_parts(src: &str) -> (Expression, Statement, Option<Statement>) {
    match statement_kind(src) {
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => (condition, *then_branch, else_branch.map(|b| *b)),
        other => panic!("`{src}` should be an if, got {other:?}"),
    }
}

fn for_parts(src: &str) -> (ForInit, Option<Expression>, Option<ForStep>) {
    match statement_kind(src) {
        StatementKind::For {
            init,
            condition,
            step,
            ..
        } => (init, condition, step),
        other => panic!("`{src}` should be a for, got {other:?}"),
    }
}

#[test]
fn an_expression_followed_by_a_semicolon_is_an_expression_statement() {
    assert_eq!(
        statement_kind("1 + 2;"),
        StatementKind::ExpressionStatement(Expression {
            kind: ExpressionKind::BinaryOp {
                op: Operator::Plus,
                left: Box::new(number(1.0, 1, 1)),
                right: Box::new(number(2.0, 5, 1)),
            },
            span: at(1, 1, 5),
        })
    );
}

#[test]
fn an_expression_statement_span_reaches_the_semicolon() {
    assert_eq!(statement("1 + 2;").span, at(1, 1, 6));
}

#[test]
fn an_expression_statement_requires_a_semicolon() {
    assert_eq!(
        error("1 + 2").message,
        "expected: punctuation `;`, got: end of input"
    );
}

#[test]
fn an_assignment_splits_into_a_target_and_a_value() {
    assert_eq!(
        statement_kind("x = 1;"),
        StatementKind::Assign {
            target: ident("x", 1, 1),
            value: number(1.0, 5, 1),
        }
    );
}

#[test]
fn an_assignment_span_reaches_the_semicolon() {
    assert_eq!(statement("x = 1;").span, at(1, 1, 6));
}

#[test]
fn the_value_of_an_assignment_may_be_any_expression() {
    let assigned = statement_kind("x = a && b;");

    match assigned {
        StatementKind::Assign { value, .. } => {
            assert!(matches!(value.kind, ExpressionKind::BinaryOp { .. }));
        }
        other => panic!("expected an assignment, got {other:?}"),
    }
}

#[test]
fn the_target_of_an_assignment_is_not_checked_for_assignability() {
    assert_eq!(
        statement_kind("1 = 2;"),
        StatementKind::Assign {
            target: number(1.0, 1, 1),
            value: number(2.0, 5, 1),
        },
        "the parser accepts any expression as a target; rejecting non-places is a later pass"
    );
}

#[test]
fn assignment_does_not_chain() {
    assert_eq!(
        error("a = b = c;").message,
        "expected: punctuation `;`, got: operator `=`"
    );
}

#[test]
fn an_empty_block_holds_nothing() {
    assert_eq!(block_of("{ }"), vec![]);
}

#[test]
fn a_block_holds_declarations_and_statements_in_order() {
    let declarations = block_of("{ let x: Number = 1; x = 2; }");

    assert_eq!(declarations.len(), 2);
    assert!(matches!(declarations[0].kind, DeclarationKind::Let { .. }));
    assert!(matches!(
        declarations[1].kind,
        DeclarationKind::Statement(_)
    ));
}

#[test]
fn blocks_nest() {
    let outer = block_of("{ { } }");

    assert_eq!(outer.len(), 1);
    assert!(matches!(
        outer[0].kind,
        DeclarationKind::Statement(Statement {
            kind: StatementKind::Block(_),
            ..
        })
    ));
}

#[test]
fn a_block_span_spans_both_braces() {
    assert_eq!(statement("{ }").span, at(1, 1, 3));
}

#[test]
fn an_unclosed_block_reports_the_missing_brace_at_the_end_of_input() {
    assert_eq!(
        error("{ x = 1;").message,
        "expected: punctuation `}`, got: end of input"
    );
}

#[test]
fn an_if_without_an_else_leaves_the_else_branch_empty() {
    let (condition, then_branch, else_branch) = if_parts("if (x) { }");

    assert_eq!(condition, ident("x", 5, 1));
    assert!(matches!(then_branch.kind, StatementKind::Block(_)));
    assert!(else_branch.is_none());
}

#[test]
fn an_if_with_an_else_keeps_both_branches() {
    let (_, then_branch, else_branch) = if_parts("if (x) a = 1; else a = 2;");

    assert!(matches!(then_branch.kind, StatementKind::Assign { .. }));
    assert!(matches!(
        else_branch.map(|b| b.kind),
        Some(StatementKind::Assign { .. })
    ));
}

#[test]
fn a_dangling_else_binds_to_the_nearest_if() {
    let (_, then_branch, else_branch) = if_parts("if (a) if (b) x = 1; else x = 2;");

    assert!(
        else_branch.is_none(),
        "the else belongs to the inner if, not the outer one"
    );
    match then_branch.kind {
        StatementKind::If { else_branch, .. } => assert!(else_branch.is_some()),
        other => panic!("the outer then-branch should be the inner if, got {other:?}"),
    }
}

#[test]
fn an_if_span_ends_at_whichever_branch_it_has() {
    assert_eq!(statement("if (x) { }").span, at(1, 1, 10));
    assert_eq!(statement("if (x) { } else { }").span, at(1, 1, 19));
}

#[test]
fn an_if_requires_a_parenthesized_condition() {
    assert_eq!(
        error("if x { }").message,
        "expected: punctuation `(`, got: identifier `x`"
    );
    assert_eq!(
        error("if (x { }").message,
        "expected: punctuation `)`, got: punctuation `{`"
    );
}

#[test]
fn a_while_keeps_its_condition_and_body() {
    match statement_kind("while (a < b) { }") {
        StatementKind::While { condition, body } => {
            assert!(matches!(condition.kind, ExpressionKind::BinaryOp { .. }));
            assert!(matches!(body.kind, StatementKind::Block(_)));
        }
        other => panic!("expected a while, got {other:?}"),
    }
}

#[test]
fn a_while_span_runs_from_the_keyword_through_the_body() {
    assert_eq!(statement("while (x) { }").span, at(1, 1, 13));
}

#[test]
fn a_while_requires_a_parenthesized_condition() {
    assert_eq!(
        error("while x { }").message,
        "expected: punctuation `(`, got: identifier `x`"
    );
}

#[test]
fn a_bare_return_carries_no_value() {
    assert_eq!(statement_kind("return;"), StatementKind::Return(None));
}

#[test]
fn a_return_with_a_value_carries_it() {
    assert_eq!(
        statement_kind("return 1;"),
        StatementKind::Return(Some(number(1.0, 8, 1)))
    );
}

#[test]
fn a_return_span_reaches_the_semicolon() {
    assert_eq!(statement("return;").span, at(1, 1, 7));
    assert_eq!(statement("return 1;").span, at(1, 1, 9));
}

#[test]
fn a_return_requires_a_semicolon() {
    assert_eq!(
        error("return 1").message,
        "expected: punctuation `;`, got: end of input"
    );
}

#[test]
fn prefix_increment_and_decrement_are_statements() {
    assert_eq!(
        statement_kind("++i;"),
        StatementKind::Increment(ident("i", 3, 1))
    );
    assert_eq!(
        statement_kind("--i;"),
        StatementKind::Decrement(ident("i", 3, 1))
    );
}

#[test]
fn an_incdec_span_runs_from_the_operator_to_the_semicolon() {
    assert_eq!(statement("++i;").span, at(1, 1, 4));
}

#[test]
fn increment_takes_a_primary_not_a_whole_expression() {
    assert_eq!(
        error("++a + b;").message,
        "expected: punctuation `;`, got: operator `+`"
    );
}

#[test]
fn postfix_increment_is_not_supported() {
    assert_eq!(
        error("i++;").message,
        "expected: punctuation `;`, got: operator `++`"
    );
}

#[test]
fn a_for_can_leave_every_clause_out() {
    let (init, condition, step) = for_parts("for (;;) { }");

    assert_eq!(init, ForInit::None);
    assert!(condition.is_none());
    assert!(step.is_none());
}

#[test]
fn a_for_can_declare_its_own_binding() {
    let (init, condition, step) = for_parts("for (let i: Number = 0; i < 3; ++i) { }");

    match init {
        ForInit::Let(declaration) => {
            assert!(matches!(declaration.kind, DeclarationKind::Let { .. }));
        }
        other => panic!("expected a let init, got {other:?}"),
    }
    assert!(condition.is_some());
    assert_eq!(step, Some(ForStep::Increment(ident("i", 34, 1))));
}

#[test]
fn a_for_can_start_from_an_assignment() {
    let (init, _, _) = for_parts("for (i = 0; i < 3; ++i) { }");

    assert_eq!(
        init,
        ForInit::Step(ForStep::Assign {
            target: ident("i", 6, 1),
            value: number(0.0, 10, 1),
        })
    );
}

#[test]
fn a_for_can_start_from_an_increment() {
    let (init, _, _) = for_parts("for (++i; i < 3; ++i) { }");

    assert_eq!(init, ForInit::Step(ForStep::Increment(ident("i", 8, 1))));
}

#[test]
fn a_for_span_runs_from_the_keyword_through_the_body() {
    assert_eq!(statement("for (;;) { }").span, at(1, 1, 12));
}

#[test]
fn a_for_step_may_be_an_assignment() {
    let (_, _, step) = for_parts("for (;; i = i + 1) { }");

    assert_eq!(
        step,
        Some(ForStep::Assign {
            target: ident("i", 9, 1),
            value: Expression {
                kind: ExpressionKind::BinaryOp {
                    op: Operator::Plus,
                    left: Box::new(ident("i", 13, 1)),
                    right: Box::new(number(1.0, 17, 1)),
                },
                span: at(1, 13, 5),
            },
        })
    );
}

#[test]
fn a_for_step_may_be_an_increment_or_a_decrement() {
    let (_, _, incremented) = for_parts("for (;; ++i) { }");
    let (_, _, decremented) = for_parts("for (;; --i) { }");

    assert_eq!(incremented, Some(ForStep::Increment(ident("i", 11, 1))));
    assert_eq!(decremented, Some(ForStep::Decrement(ident("i", 11, 1))));
}

#[test]
fn a_for_step_may_not_be_a_bare_expression() {
    assert_eq!(
        error("for (;; i) { }").message,
        "expected: operator `=`, got: punctuation `)`",
        "a step must increment, decrement, or assign"
    );
    assert_eq!(
        error("for (i; i < 3;) { }").message,
        "expected: operator `=`, got: punctuation `;`"
    );
}

#[test]
fn a_for_step_increment_takes_a_primary_not_a_whole_expression() {
    assert_eq!(
        error("for (;; ++a + b) { }").message,
        "expected: punctuation `)`, got: operator `+`"
    );
}

#[test]
fn a_for_requires_both_of_its_semicolons() {
    assert_eq!(
        error("for () { }").message,
        "cannot parse: punctuation `)` on 1:6"
    );
    assert_eq!(
        error("for (;) { }").message,
        "cannot parse: punctuation `)` on 1:7"
    );
}

#[test]
fn a_for_requires_a_body() {
    assert_eq!(
        error("for (;;)").message,
        "cannot parse: end of input on 1:9"
    );
}
