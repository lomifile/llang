use super::{at, declaration, declaration_kind, error};
use ast::statement::{DeclarationKind, ExpressionKind, FunctionParam, StatementKind, Type};

fn let_parts(src: &str) -> (String, Type, ExpressionKind) {
    match declaration_kind(src) {
        DeclarationKind::Let { name, ty, init } => (name, ty, init.kind),
        other => panic!("`{src}` should be a let, got {other:?}"),
    }
}

fn function_parts(src: &str) -> (String, Vec<FunctionParam>, Type, StatementKind) {
    match declaration_kind(src) {
        DeclarationKind::Function {
            name,
            params,
            return_type,
            body,
        } => (name, params, return_type, body.kind),
        other => panic!("`{src}` should be a function, got {other:?}"),
    }
}

fn param(name: &str, param_type: Type) -> FunctionParam {
    FunctionParam {
        name: name.to_string(),
        param_type,
    }
}

#[test]
fn a_let_binds_a_name_a_type_and_an_initializer() {
    assert_eq!(
        let_parts("let count: Number = 1;"),
        (
            "count".to_string(),
            Type::Number,
            ExpressionKind::LiteralNumber(1.0)
        )
    );
}

#[test]
fn a_const_is_the_same_shape_under_a_different_kind() {
    assert_eq!(
        declaration_kind("const greeting: String = \"hi\";"),
        DeclarationKind::Const {
            name: "greeting".to_string(),
            ty: Type::String,
            init: ast::statement::Expression {
                kind: ExpressionKind::LiteralString("hi".to_string()),
                span: at(1, 26, 2),
            },
        }
    );
}

#[test]
fn a_binding_accepts_a_full_expression_as_its_initializer() {
    let (_, _, init) = let_parts("let total: Number = 1 + 2 * 3;");

    assert!(
        matches!(init, ExpressionKind::BinaryOp { .. }),
        "the initializer should keep its whole expression tree, got {init:?}"
    );
}

#[test]
fn a_binding_span_runs_from_the_keyword_to_the_semicolon() {
    assert_eq!(declaration("let x: Number = 1;").span, at(1, 1, 18));
}

#[test]
fn a_binding_requires_a_type_annotation() {
    assert_eq!(
        error("let x = 1;").message,
        "expected: punctuation `:`, got: operator `=`"
    );
}

#[test]
fn a_binding_requires_an_initializer() {
    assert_eq!(
        error("let x: Number;").message,
        "expected: operator `=`, got: punctuation `;`"
    );
}

#[test]
fn a_binding_requires_a_name() {
    let failed = error("let 1: Number = 1;");

    assert_eq!(failed.message, "expected variable name");
    assert_eq!(failed.span, at(1, 5, 1));
}

#[test]
fn a_binding_requires_a_terminating_semicolon() {
    assert_eq!(
        error("let x: Number = 1").message,
        "expected: punctuation `;`, got: end of input"
    );
}

#[test]
fn a_function_with_no_parameters_parses() {
    assert_eq!(
        function_parts("function main(): Void { }"),
        (
            "main".to_string(),
            vec![],
            Type::Void,
            StatementKind::Block(vec![])
        )
    );
}

#[test]
fn a_function_keeps_its_parameters_in_order() {
    let (_, params, return_type, _) =
        function_parts("function add(a: Number, b: Array[String]): Number { }");

    assert_eq!(
        params,
        vec![
            param("a", Type::Number),
            param("b", Type::Array(Box::new(Type::String))),
        ]
    );
    assert_eq!(return_type, Type::Number);
}

#[test]
fn a_function_body_holds_its_declarations() {
    let (_, _, _, body) = function_parts("function f(): Void { let x: Number = 1; }");

    match body {
        StatementKind::Block(declarations) => assert_eq!(declarations.len(), 1),
        other => panic!("a body should be a block, got {other:?}"),
    }
}

#[test]
fn a_function_span_runs_from_the_keyword_to_the_closing_brace() {
    assert_eq!(declaration("function f(): Void { }").span, at(1, 1, 22));
}

#[test]
fn a_function_requires_a_parameter_list() {
    assert_eq!(
        error("function f: Void { }").message,
        "expected: punctuation `(`, got: punctuation `:`"
    );
}

#[test]
fn a_function_requires_a_return_type() {
    assert_eq!(
        error("function f() { }").message,
        "expected: punctuation `:`, got: punctuation `{`"
    );
}

#[test]
fn a_function_requires_a_body() {
    assert_eq!(
        error("function f(): Void;").message,
        "expected: punctuation `{`, got: punctuation `;`"
    );
}

#[test]
fn a_parameter_requires_a_type() {
    assert_eq!(
        error("function f(a): Void { }").message,
        "expected: punctuation `:`, got: punctuation `)`"
    );
}

#[test]
fn a_trailing_comma_in_a_parameter_list_is_rejected() {
    assert_eq!(
        error("function f(a: Number,): Void { }").message,
        "expected param name"
    );
}

#[test]
fn a_missing_comma_between_parameters_is_rejected() {
    assert_eq!(
        error("function f(a: Number b: Number): Void { }").message,
        "expected: punctuation `)`, got: identifier `b`"
    );
}

#[test]
fn anything_that_is_not_a_binding_or_function_becomes_a_statement() {
    assert!(matches!(
        declaration_kind("x = 1;"),
        DeclarationKind::Statement(_)
    ));
    assert!(matches!(
        declaration_kind("{ }"),
        DeclarationKind::Statement(_)
    ));
}

#[test]
fn a_wrapped_statement_borrows_the_statement_span() {
    let wrapped = declaration("return;");

    match wrapped.kind {
        DeclarationKind::Statement(inner) => assert_eq!(inner.span, wrapped.span),
        other => panic!("expected a statement, got {other:?}"),
    }
}
