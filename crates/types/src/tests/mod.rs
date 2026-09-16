use crate::checker::Checker;
use crate::error::TypeError;
use lexer::lexer::Lexer;
use parser::parser::Parser;

mod ill_typed;
mod well_typed;

fn check(source: &str) -> Result<(), TypeError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .unwrap_or_else(|e| panic!("`{source}` should lex, got {e:?}"));

    let declarations = Parser::new(tokens)
        .parse_program()
        .unwrap_or_else(|e| panic!("`{source}` should parse, got {}: {}", e.span, e.message));

    Checker::new().check_program(&declarations)
}

fn ok(source: &str) {
    if let Err(e) = check(source) {
        panic!(
            "`{source}` should type-check, got {}: {}",
            e.span, e.message
        );
    }
}

fn err(source: &str) -> TypeError {
    check(source).expect_err(&format!("`{source}` should not type-check"))
}

fn err_contains(source: &str, needle: &str) {
    let error = err(source);
    assert!(
        error.message.contains(needle),
        "`{source}` should fail with a message containing `{needle}`, got `{}`",
        error.message
    );
}
