use super::ok;

#[test]
fn an_empty_program_checks() {
    ok("");
}

#[test]
fn the_sample_program_checks() {
    ok("const limit: Number = 10;
let total: Number = 0;

function add(a: Number, b: Number): Number {
    return a + b;
}

function run(): Void {
    let values: Array[Number] = [1, 2, 3];
    let i: Number = 0;

    while (i < 3) {
        total = add(total, values[i]);
        ++i;
    }

    if (total > limit) {
        total = limit;
    } else {
        total = 0;
    }

    for (let j: Number = 0; j < 3; ++j) {
        total = add(total, 1);
    }
}");
}

#[test]
fn a_block_may_shadow_an_outer_binding_with_a_different_type() {
    ok("let x: Number = 1; { let x: String = \"a\"; }");
}

#[test]
fn a_function_may_call_one_defined_later() {
    ok("function first(): Number { return second(); }
function second(): Number { return 1; }");
}

#[test]
fn a_let_binding_is_assignable() {
    ok("let x: Number = 1; x = 2;");
}

#[test]
fn an_element_of_a_const_array_is_assignable() {
    ok("const a: Array[Number] = [1]; a[0] = 2;");
}

#[test]
fn a_void_function_may_return_without_a_value() {
    ok("function f(): Void { return; }");
}

#[test]
fn a_parameter_is_in_scope_in_the_body() {
    ok("function f(s: String): String { return s; }");
}

#[test]
fn a_nested_array_type_checks_elementwise() {
    ok("let grid: Array[Array[Number]] = [[1, 2], [3]];");
}

#[test]
fn comparison_and_logic_produce_booleans() {
    ok("let b: Boolean = 1 < 2 && !(3 === 4);");
}
