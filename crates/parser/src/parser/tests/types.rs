use super::{at, source};
use ast::statement::Type;
use crate::error::ParserError;

fn ty(src: &str) -> Type {
    source(src)
        .parse_type()
        .unwrap_or_else(|e| panic!("`{src}` should be a type, got {}", e.message))
}

fn ty_error(src: &str) -> ParserError {
    source(src)
        .parse_type()
        .expect_err(&format!("`{src}` should not be a type"))
}

#[test]
fn every_primitive_name_maps_to_its_type() {
    assert_eq!(ty("Number"), Type::Number);
    assert_eq!(ty("String"), Type::String);
    assert_eq!(ty("Boolean"), Type::Boolean);
    assert_eq!(ty("Object"), Type::Object);
    assert_eq!(ty("Void"), Type::Void);
}

#[test]
fn null_is_spelled_with_the_keyword_not_an_identifier() {
    assert_eq!(ty("Null"), Type::Null);
}

#[test]
fn an_array_wraps_its_element_type() {
    assert_eq!(ty("Array[Number]"), Type::Array(Box::new(Type::Number)));
}

#[test]
fn arrays_nest() {
    assert_eq!(
        ty("Array[Array[String]]"),
        Type::Array(Box::new(Type::Array(Box::new(Type::String))))
    );
}

#[test]
fn an_unknown_name_is_rejected_at_its_own_span() {
    let error = ty_error("Widget");

    assert_eq!(error.message, "unknown type: Widget");
    assert_eq!(error.span, at(1, 1, 6));
}

#[test]
fn type_names_are_case_sensitive() {
    assert_eq!(ty_error("number").message, "unknown type: number");
}

#[test]
fn a_token_that_is_not_a_name_reports_expected_a_type() {
    assert_eq!(ty_error("123").message, "expected a type");
    assert_eq!(ty_error("[").message, "expected a type");
    assert_eq!(ty_error("let").message, "expected a type");
    assert_eq!(ty_error("").message, "expected a type");
}

#[test]
fn an_array_without_an_element_type_reports_the_missing_bracket() {
    assert_eq!(
        ty_error("Array").message,
        "expected: punctuation `[`, got: end of input"
    );
}

#[test]
fn an_unclosed_array_reports_the_missing_bracket() {
    assert_eq!(
        ty_error("Array[Number").message,
        "expected: punctuation `]`, got: end of input"
    );
}
