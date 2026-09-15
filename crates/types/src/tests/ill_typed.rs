use super::{err, err_contains};

#[test]
fn an_initializer_must_match_its_annotation() {
    err_contains("let x: Number = \"s\";", "type mismatch");
}

#[test]
fn an_if_condition_must_be_boolean() {
    err_contains("if (5) { }", "if condition must be Boolean");
}

#[test]
fn a_while_condition_must_be_boolean() {
    err_contains("while (5) { }", "while condition must be boolean");
}

#[test]
fn a_for_condition_must_be_boolean() {
    err_contains(
        "for (let i: Number = 0; i; ++i) { }",
        "for condition must be Boolean",
    );
}

#[test]
fn an_argument_must_match_its_parameter_type() {
    err_contains(
        "function f(a: Number): Void { } f(\"s\");",
        "argument type mismatch",
    );
}

#[test]
fn a_call_must_pass_the_right_number_of_arguments() {
    err_contains("function f(a: Number): Void { } f(1, 2);", "expects");
    err_contains("function f(a: Number): Void { } f();", "expects");
}

#[test]
fn an_undefined_function_cannot_be_called() {
    err_contains("f(1);", "unknown function");
}

#[test]
fn a_const_binding_cannot_be_reassigned() {
    err_contains("const x: Number = 1; x = 2;", "cannot assign to const");
}

#[test]
fn a_function_must_return_its_declared_type() {
    err_contains(
        "function f(): Number { return \"s\"; }",
        "return type mismatch",
    );
}

#[test]
fn a_non_void_function_cannot_return_without_a_value() {
    err_contains("function f(): Number { return; }", "must return");
}

#[test]
fn a_void_function_cannot_return_a_value() {
    err_contains(
        "function f(): Void { return 5; }",
        "void function cannot return a value",
    );
}

#[test]
fn an_undefined_variable_cannot_be_read() {
    err_contains("let y: Number = x;", "unknown variable");
}

#[test]
fn arithmetic_requires_numbers() {
    err_contains("let x: Number = 1 + \"s\";", "should be numbers");
}

#[test]
fn equality_requires_matching_types() {
    err("let b: Boolean = 1 === \"s\";");
}

#[test]
fn an_array_literal_must_be_homogeneous() {
    err_contains("let a: Array[Number] = [1, \"two\"];", "same type");
}

#[test]
fn an_empty_array_literal_has_no_inferable_type() {
    err_contains(
        "let a: Array[Number] = [];",
        "cannot infer type of empty array",
    );
}

#[test]
fn a_non_array_cannot_be_indexed() {
    err_contains("let s: String = \"a\"; let c: String = s[0];", "cannot index");
}

#[test]
fn an_index_must_be_a_number() {
    err_contains(
        "let a: Array[Number] = [1]; let x: Number = a[\"k\"];",
        "array index must be Number",
    );
}

#[test]
fn a_name_cannot_be_defined_twice_in_one_scope() {
    err_contains("let x: Number = 1; let x: Number = 2;", "redefinition");
}

#[test]
fn a_function_cannot_be_defined_twice() {
    err_contains(
        "function f(): Void { } function f(): Void { }",
        "redefinit",
    );
}

#[test]
fn return_is_not_allowed_at_the_top_level() {
    err("return 5;");
}

#[test]
fn increment_requires_a_number() {
    err_contains("let s: String = \"a\"; ++s;", "require a Number");
}

#[test]
fn an_assignment_must_match_the_target_type() {
    err_contains("let x: Number = 1; x = \"s\";", "assignment mismatch");
}

#[test]
fn a_binding_from_a_closed_block_is_out_of_scope() {
    err_contains("{ let x: Number = 1; } x = 2;", "unknown variable");
}
