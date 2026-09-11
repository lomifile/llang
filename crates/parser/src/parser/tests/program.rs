use super::{error, program, source};
use ast::statement::DeclarationKind;

#[test]
fn an_empty_program_has_no_declarations() {
    assert_eq!(program(""), vec![]);
    assert_eq!(program("   \n\n  "), vec![]);
}

#[test]
fn a_comment_only_program_has_no_declarations() {
    assert_eq!(program("// nothing to see\n"), vec![]);
}

#[test]
fn a_program_keeps_its_declarations_in_source_order() {
    let declarations = program("let a: Number = 1;\nfunction f(): Void { }\na = 2;");

    assert!(matches!(declarations[0].kind, DeclarationKind::Let { .. }));
    assert!(matches!(
        declarations[1].kind,
        DeclarationKind::Function { .. }
    ));
    assert!(matches!(
        declarations[2].kind,
        DeclarationKind::Statement(_)
    ));
    assert_eq!(declarations.len(), 3);
}

#[test]
fn a_program_consumes_every_token_up_to_the_end() {
    let mut p = source("let a: Number = 1; a = 2;");

    p.parse_program().expect("both should parse");

    assert!(p.is_at_end());
}

#[test]
fn a_program_stops_at_the_first_error() {
    let failed = error("let a: Number = 1;\nlet: Number = 2;\nlet c: Number = 3;");

    assert_eq!(failed.message, "expected variable name");
    assert_eq!(failed.span.line, 2);
}

#[test]
fn spans_track_the_line_the_declaration_starts_on() {
    let declarations = program("let a: Number = 1;\n\nlet b: Number = 2;");

    assert_eq!(declarations[0].span.line, 1);
    assert_eq!(declarations[1].span.line, 3);
    assert_eq!(declarations[1].span.col, 1);
}

const SAMPLE: &str = r#"const limit: Number = 10;
let total: Number = 0;

function add(a: Number, b: Number): Number {
    return a + b;
}

for (let i: Number = 1; i <= limit; ++i) {
    total = total + i;
}

let count: Number = 0;
while (count !== limit) {
    ++count;
}

if (count === limit) {
    total = 0;
}
"#;

#[test]
fn the_sample_program_parses() {
    let declarations = program(SAMPLE);

    assert_eq!(declarations.len(), 7);
}
