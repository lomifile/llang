#![allow(clippy::float_cmp)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use chunk::types::{Chunk, Function, Op, Value};

use crate::types::{RuntimeError, Vm};

fn chunk(code: Vec<Op>, constants: Vec<Value>) -> Chunk {
    let lines: Vec<u32> = (1..).take(code.len()).collect();
    Chunk {
        code,
        constants,
        lines,
    }
}

fn run(code: Vec<Op>, constants: Vec<Value>) -> (Vm, Result<(), RuntimeError>) {
    let mut vm = Vm::new();
    let result = vm.interpret(chunk(code, constants));
    (vm, result)
}

fn eval(code: Vec<Op>, constants: Vec<Value>) -> Value {
    let (vm, result) = run(code, constants);
    result.expect("expected the program to run without a runtime error");
    assert_eq!(vm.stack.len(), 1, "expected a single value on the stack");
    vm.stack[0].clone()
}

fn eval_number(code: Vec<Op>, constants: Vec<Value>) -> f64 {
    match eval(code, constants) {
        Value::Number(n) => n,
        other => panic!("expected a number, got {other:?}"),
    }
}

fn eval_bool(code: Vec<Op>, constants: Vec<Value>) -> bool {
    match eval(code, constants) {
        Value::Bool(b) => b,
        other => panic!("expected a bool, got {other:?}"),
    }
}

fn error(code: Vec<Op>, constants: Vec<Value>) -> RuntimeError {
    let (_, result) = run(code, constants);
    result.expect_err("expected a runtime error")
}

fn binary(op: Op) -> Vec<Op> {
    vec![Op::Constant(0), Op::Constant(1), op]
}

fn numbers(a: f64, b: f64) -> Vec<Value> {
    vec![Value::Number(a), Value::Number(b)]
}

fn booleans(a: bool, b: bool) -> Vec<Value> {
    vec![Value::Bool(a), Value::Bool(b)]
}

#[test]
fn empty_chunk_runs_and_leaves_an_empty_stack() {
    let (vm, result) = run(vec![], vec![]);
    assert!(result.is_ok());
    assert!(vm.stack.is_empty());
}

#[test]
fn constant_pushes_the_value_at_the_index() {
    let value = eval(
        vec![Op::Constant(1)],
        vec![Value::Number(1.0), Value::Str(Rc::from("hello"))],
    );
    assert_eq!(value, Value::Str(Rc::from("hello")));
}

#[test]
fn literal_ops_push_their_values() {
    assert_eq!(eval(vec![Op::Null], vec![]), Value::Null);
    assert_eq!(eval(vec![Op::True], vec![]), Value::Bool(true));
    assert_eq!(eval(vec![Op::False], vec![]), Value::Bool(false));
}

#[test]
fn add_sums_the_operands() {
    assert_eq!(eval_number(binary(Op::Add), numbers(2.0, 3.0)), 5.0);
}

#[test]
fn sub_subtracts_the_top_from_the_one_below() {
    assert_eq!(eval_number(binary(Op::Sub), numbers(10.0, 4.0)), 6.0);
}

#[test]
fn mul_multiplies_the_operands() {
    assert_eq!(eval_number(binary(Op::Mul), numbers(6.0, 7.0)), 42.0);
}

#[test]
fn mod_takes_the_remainder_in_operand_order() {
    assert_eq!(eval_number(binary(Op::Mod), numbers(10.0, 3.0)), 1.0);
}

#[test]
fn div_divides_the_operands_in_order() {
    assert_eq!(eval_number(binary(Op::Div), numbers(9.0, 2.0)), 4.5);
}

#[test]
fn div_by_zero_is_a_runtime_error() {
    let err = error(binary(Op::Div), numbers(1.0, 0.0));
    assert_eq!(err.message, "division by zero");
    assert_eq!(err.line, 3);
}

#[test]
fn negate_flips_the_sign() {
    assert_eq!(
        eval_number(vec![Op::Constant(0), Op::Negate], vec![Value::Number(3.0)]),
        -3.0
    );
}

#[test]
fn not_flips_the_boolean() {
    assert!(!eval_bool(vec![Op::True, Op::Not], vec![]));
    assert!(eval_bool(vec![Op::False, Op::Not], vec![]));
}

#[test]
fn equal_compares_values_of_the_same_type() {
    assert!(eval_bool(binary(Op::Equal), numbers(1.0, 1.0)));
    assert!(!eval_bool(binary(Op::Equal), numbers(1.0, 2.0)));
    assert!(eval_bool(vec![Op::Null, Op::Null, Op::Equal], vec![]));
}

#[test]
fn equal_is_false_across_types() {
    assert!(!eval_bool(vec![Op::Null, Op::False, Op::Equal], vec![]));
}

#[test]
fn greater_and_less_respect_operand_order() {
    assert!(eval_bool(binary(Op::Greater), numbers(3.0, 2.0)));
    assert!(!eval_bool(binary(Op::Greater), numbers(2.0, 3.0)));
    assert!(!eval_bool(binary(Op::Greater), numbers(2.0, 2.0)));

    assert!(eval_bool(binary(Op::Less), numbers(2.0, 3.0)));
    assert!(!eval_bool(binary(Op::Less), numbers(3.0, 2.0)));
    assert!(!eval_bool(binary(Op::Less), numbers(2.0, 2.0)));
}

#[test]
fn and_covers_the_truth_table() {
    for (a, b) in [(true, true), (true, false), (false, true), (false, false)] {
        assert_eq!(
            eval_bool(binary(Op::And), booleans(a, b)),
            a && b,
            "{a} && {b}"
        );
    }
}

#[test]
fn or_covers_the_truth_table() {
    for (a, b) in [(true, true), (true, false), (false, true), (false, false)] {
        assert_eq!(
            eval_bool(binary(Op::Or), booleans(a, b)),
            a || b,
            "{a} || {b}"
        );
    }
}

#[test]
fn pop_discards_the_top_of_the_stack() {
    let (vm, result) = run(
        vec![Op::Constant(0), Op::Constant(1), Op::Pop],
        numbers(1.0, 2.0),
    );
    assert!(result.is_ok());
    assert_eq!(vm.stack, vec![Value::Number(1.0)]);
}

#[test]
fn pop_on_an_empty_stack_underflows() {
    let err = error(vec![Op::Pop], vec![]);
    assert_eq!(err.message, "stack underflow");
    assert_eq!(err.line, 1);
}

#[test]
fn arithmetic_on_a_non_number_is_a_type_error() {
    let err = error(vec![Op::True, Op::True, Op::Add], vec![]);
    assert_eq!(err.message, "expected a number");
    assert_eq!(err.line, 3);
}

#[test]
fn logic_on_a_non_boolean_is_a_type_error() {
    let err = error(
        vec![Op::Constant(0), Op::Constant(1), Op::And],
        numbers(1.0, 2.0),
    );
    assert_eq!(err.message, "expected a bool");
    assert_eq!(err.line, 3);
}

#[test]
fn binary_arithmetic_underflows_when_an_operand_is_missing() {
    let err = error(vec![Op::Constant(0), Op::Add], vec![Value::Number(1.0)]);
    assert_eq!(err.message, "stack underflow");
    assert_eq!(err.line, 2);
}

#[test]
fn a_nested_expression_evaluates_left_to_right() {
    let value = eval_number(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::Add,
            Op::Constant(2),
            Op::Mul,
            Op::Negate,
        ],
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(4.0)],
    );
    assert_eq!(value, -12.0);
}

fn function(name: &str, arity: usize, code: Vec<Op>, constants: Vec<Value>) -> Value {
    Value::Function(Rc::new(Function {
        name: Rc::from(name),
        arity,
        chunk: chunk(code, constants),
    }))
}

fn add_function() -> Value {
    function(
        "add",
        2,
        vec![Op::GetLocal(0), Op::GetLocal(1), Op::Add, Op::Return],
        vec![],
    )
}

fn sample_program() -> (Vec<Op>, Vec<Value>) {
    let constants = vec![
        Value::Str(Rc::from("add")),
        add_function(),
        Value::Number(0.0),
        Value::Str(Rc::from("total")),
        Value::Number(1.0),
        Value::Number(5.0),
        Value::Str(Rc::from("result")),
        Value::Number(15.0),
    ];

    let code = vec![
        Op::Constant(1),
        Op::DefineGlobal(0),
        Op::Constant(2),
        Op::DefineGlobal(3),
        Op::Constant(4),
        Op::GetLocal(0),
        Op::Constant(5),
        Op::Greater,
        Op::Not,
        Op::JumpIfFalse(20),
        Op::GetGlobal(0),
        Op::GetGlobal(3),
        Op::GetLocal(0),
        Op::Call(2),
        Op::SetGlobal(3),
        Op::GetLocal(0),
        Op::Constant(4),
        Op::Add,
        Op::SetLocal(0),
        Op::Loop(5),
        Op::Pop,
        Op::Constant(2),
        Op::DefineGlobal(6),
        Op::GetGlobal(3),
        Op::Constant(7),
        Op::Less,
        Op::Not,
        Op::JumpIfFalse(31),
        Op::GetGlobal(3),
        Op::SetGlobal(6),
        Op::Jump(33),
        Op::Constant(2),
        Op::SetGlobal(6),
    ];

    (code, constants)
}

#[test]
fn sample_program_accumulates_one_through_five_and_takes_the_then_branch() {
    let (code, constants) = sample_program();
    let (vm, result) = run(code, constants);

    result.expect("expected the sample program to run without a runtime error");

    assert_eq!(vm.globals.get("total"), Some(&Value::Number(15.0)));
    assert_eq!(vm.globals.get("result"), Some(&Value::Number(15.0)));
    assert!(matches!(vm.globals.get("add"), Some(Value::Function(_))));
    assert!(
        vm.stack.is_empty(),
        "the loop local should be popped, got {:?}",
        vm.stack
    );
}

#[test]
fn sample_program_takes_the_else_branch_when_the_total_is_below_the_threshold() {
    let (code, mut constants) = sample_program();
    constants[5] = Value::Number(3.0);

    let (vm, result) = run(code, constants);
    result.expect("expected the sample program to run without a runtime error");

    assert_eq!(vm.globals.get("total"), Some(&Value::Number(6.0)));
    assert_eq!(vm.globals.get("result"), Some(&Value::Number(0.0)));
}

#[test]
fn a_loop_whose_condition_starts_false_never_runs_its_body() {
    let (code, mut constants) = sample_program();
    constants[4] = Value::Number(10.0);

    let (vm, result) = run(code, constants);
    result.expect("expected the sample program to run without a runtime error");

    assert_eq!(vm.globals.get("total"), Some(&Value::Number(0.0)));
    assert_eq!(vm.globals.get("result"), Some(&Value::Number(0.0)));
}

#[test]
fn a_call_restores_the_caller_stack_and_leaves_only_the_return_value() {
    let (vm, result) = run(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::Constant(2),
            Op::Call(2),
        ],
        vec![add_function(), Value::Number(2.0), Value::Number(3.0)],
    );

    result.expect("expected the call to succeed");
    assert_eq!(vm.stack, vec![Value::Number(5.0)]);
}

#[test]
fn calling_with_the_wrong_argument_count_is_a_runtime_error() {
    let err = error(
        vec![Op::Constant(0), Op::Constant(1), Op::Call(1)],
        vec![add_function(), Value::Number(2.0)],
    );
    assert_eq!(err.message, "wrong argument count");
}

#[test]
fn calling_a_non_function_is_a_runtime_error() {
    let err = error(
        vec![Op::Constant(0), Op::Constant(0), Op::Call(1)],
        vec![Value::Number(1.0)],
    );
    assert_eq!(err.message, "can only call function");
}

#[test]
fn reading_an_undefined_global_is_a_runtime_error() {
    let err = error(vec![Op::GetGlobal(0)], vec![Value::Str(Rc::from("total"))]);
    assert_eq!(err.message, "undefined variable: total");
}

#[test]
fn assigning_to_an_undefined_global_is_a_runtime_error() {
    let err = error(
        vec![Op::Constant(1), Op::SetGlobal(0)],
        vec![Value::Str(Rc::from("total")), Value::Number(1.0)],
    );
    assert_eq!(err.message, "assignment to undefined variable: total");
}

#[test]
fn set_local_writes_through_to_the_slot() {
    let (vm, result) = run(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::SetLocal(0),
            Op::GetLocal(0),
        ],
        vec![Value::Number(1.0), Value::Number(9.0)],
    );

    result.expect("expected the program to succeed");
    assert_eq!(vm.stack, vec![Value::Number(9.0), Value::Number(9.0)]);
}

#[test]
fn a_top_level_return_ends_the_program_without_underflowing() {
    let (vm, result) = run(vec![Op::Constant(0), Op::Return], vec![Value::Number(1.0)]);

    result.expect("expected a top-level return to end the program cleanly");
    assert!(vm.stack.is_empty());
    assert!(vm.frames.is_empty());
}

#[test]
fn a_top_level_return_skips_the_rest_of_the_script() {
    let (vm, result) = run(
        vec![
            Op::Constant(1),
            Op::DefineGlobal(0),
            Op::Null,
            Op::Return,
            Op::Constant(1),
            Op::DefineGlobal(2),
        ],
        vec![
            Value::Str(Rc::from("total")),
            Value::Number(0.0),
            Value::Str(Rc::from("result")),
        ],
    );

    result.expect("expected a top-level return to end the program cleanly");

    assert_eq!(vm.globals.get("total"), Some(&Value::Number(0.0)));
    assert_eq!(vm.globals.get("result"), None);
}

#[test]
fn an_error_on_a_chunk_without_line_information_reports_line_zero() {
    let mut vm = Vm::new();
    let result = vm.interpret(Chunk {
        code: vec![Op::Pop],
        constants: vec![],
        lines: vec![],
    });

    let err = result.expect_err("expected a stack underflow");
    assert_eq!(err.message, "stack underflow");
    assert_eq!(err.line, 0);
}

#[test]
fn an_error_inside_a_call_reports_the_line_in_the_callee() {
    let callee = function("boom", 0, vec![Op::True, Op::Negate], vec![]);
    let err = error(vec![Op::Constant(0), Op::Call(0)], vec![callee]);

    assert_eq!(err.message, "expected a number");
    assert_eq!(err.line, 2);
}

fn array(items: Vec<Value>) -> Rc<RefCell<Vec<Value>>> {
    Rc::new(RefCell::new(items))
}

fn elements(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.borrow().clone(),
        other => panic!("expected an array, got {other:?}"),
    }
}

fn fields(value: &Value) -> HashMap<String, Value> {
    match value {
        Value::Object(fields) => fields.borrow().clone(),
        other => panic!("expected an object, got {other:?}"),
    }
}

#[test]
fn build_array_collects_the_elements_in_push_order() {
    let value = eval(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::Constant(2),
            Op::BuildArray(3),
        ],
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
    );

    assert_eq!(
        elements(&value),
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
    );
}

#[test]
fn build_array_of_zero_elements_is_empty() {
    assert_eq!(elements(&eval(vec![Op::BuildArray(0)], vec![])), vec![]);
}

#[test]
fn build_array_consumes_only_its_own_elements() {
    let (vm, result) = run(
        vec![Op::Constant(0), Op::Constant(1), Op::BuildArray(1)],
        vec![Value::Number(1.0), Value::Number(2.0)],
    );

    result.expect("expected the program to succeed");
    assert_eq!(vm.stack.len(), 2);
    assert_eq!(vm.stack[0], Value::Number(1.0));
    assert_eq!(elements(&vm.stack[1]), vec![Value::Number(2.0)]);
}

#[test]
fn build_object_pairs_each_key_with_the_value_above_it() {
    let value = eval(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::Constant(2),
            Op::Constant(3),
            Op::BuildObject(2),
        ],
        vec![
            Value::Str(Rc::from("a")),
            Value::Number(1.0),
            Value::Str(Rc::from("b")),
            Value::Number(2.0),
        ],
    );

    let fields = fields(&value);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields.get("a"), Some(&Value::Number(1.0)));
    assert_eq!(fields.get("b"), Some(&Value::Number(2.0)));
}

#[test]
fn build_object_of_zero_pairs_is_empty() {
    assert!(fields(&eval(vec![Op::BuildObject(0)], vec![])).is_empty());
}

#[test]
fn build_object_rejects_a_non_string_key() {
    let err = error(
        vec![Op::Constant(0), Op::Constant(0), Op::BuildObject(1)],
        vec![Value::Number(1.0)],
    );
    assert_eq!(err.message, "cannot find key");
}

#[test]
fn index_get_reads_the_element_at_the_index() {
    let value = eval(
        vec![Op::Constant(0), Op::Constant(1), Op::IndexGet],
        vec![
            Value::Array(array(vec![Value::Number(10.0), Value::Number(20.0)])),
            Value::Number(1.0),
        ],
    );

    assert_eq!(value, Value::Number(20.0));
}

#[test]
fn index_get_past_the_end_reads_null() {
    let value = eval(
        vec![Op::Constant(0), Op::Constant(1), Op::IndexGet],
        vec![
            Value::Array(array(vec![Value::Number(10.0)])),
            Value::Number(4.0),
        ],
    );

    assert_eq!(value, Value::Null);
}

#[test]
fn index_get_on_a_non_array_is_a_runtime_error() {
    let err = error(
        vec![Op::Constant(0), Op::Constant(0), Op::IndexGet],
        vec![Value::Number(1.0)],
    );
    assert_eq!(err.message, "not an array");
}

#[test]
fn a_non_integer_index_is_a_runtime_error() {
    for (index, message) in [
        (1.5, "invalid array index: 1.5"),
        (-1.0, "invalid array index: -1"),
        (f64::NAN, "invalid array index: NaN"),
        (f64::INFINITY, "invalid array index: inf"),
    ] {
        let err = error(
            vec![Op::Constant(0), Op::Constant(1), Op::IndexGet],
            vec![
                Value::Array(array(vec![Value::Number(10.0)])),
                Value::Number(index),
            ],
        );
        assert_eq!(err.message, message);
    }
}

#[test]
fn index_set_replaces_the_element_in_place() {
    let items = array(vec![Value::Number(10.0), Value::Number(20.0)]);

    let (vm, result) = run(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::Constant(2),
            Op::IndexSet,
        ],
        vec![
            Value::Array(Rc::clone(&items)),
            Value::Number(0.0),
            Value::Number(99.0),
        ],
    );

    result.expect("expected the assignment to succeed");
    assert_eq!(
        *items.borrow(),
        vec![Value::Number(99.0), Value::Number(20.0)]
    );
    assert!(vm.stack.is_empty());
}

#[test]
fn index_set_past_the_end_is_a_runtime_error() {
    let err = error(
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::Constant(2),
            Op::IndexSet,
        ],
        vec![
            Value::Array(array(vec![Value::Number(10.0)])),
            Value::Number(3.0),
            Value::Number(99.0),
        ],
    );

    assert_eq!(err.message, "array out of bounds");
}

#[test]
fn member_get_reads_the_named_field() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::Number(7.0));

    let value = eval(
        vec![Op::Constant(1), Op::MemberGet(0)],
        vec![
            Value::Str(Rc::from("name")),
            Value::Object(Rc::new(RefCell::new(map))),
        ],
    );

    assert_eq!(value, Value::Number(7.0));
}

#[test]
fn member_get_of_a_missing_field_reads_null() {
    let value = eval(
        vec![Op::Constant(1), Op::MemberGet(0)],
        vec![
            Value::Str(Rc::from("missing")),
            Value::Object(Rc::new(RefCell::new(HashMap::new()))),
        ],
    );

    assert_eq!(value, Value::Null);
}

#[test]
fn member_get_on_a_non_object_is_a_runtime_error() {
    let err = error(
        vec![Op::Constant(1), Op::MemberGet(0)],
        vec![Value::Str(Rc::from("name")), Value::Number(1.0)],
    );
    assert_eq!(err.message, "not an object");
}

#[test]
fn member_set_defines_and_replaces_fields_in_place() {
    let object = Rc::new(RefCell::new(HashMap::new()));
    object
        .borrow_mut()
        .insert("name".to_string(), Value::Number(1.0));

    let (vm, result) = run(
        vec![
            Op::Constant(1),
            Op::Constant(2),
            Op::MemberSet(0),
            Op::Constant(1),
            Op::Constant(3),
            Op::MemberSet(4),
        ],
        vec![
            Value::Str(Rc::from("name")),
            Value::Object(Rc::clone(&object)),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Str(Rc::from("extra")),
        ],
    );

    result.expect("expected the assignments to succeed");
    assert_eq!(object.borrow().get("name"), Some(&Value::Number(2.0)));
    assert_eq!(object.borrow().get("extra"), Some(&Value::Number(3.0)));
    assert!(vm.stack.is_empty());
}

#[test]
fn member_set_on_a_non_object_is_a_runtime_error() {
    let err = error(
        vec![Op::Constant(1), Op::Constant(1), Op::MemberSet(0)],
        vec![Value::Str(Rc::from("name")), Value::Number(1.0)],
    );
    assert_eq!(err.message, "not an object");
}

#[test]
fn build_array_with_too_few_elements_underflows_instead_of_panicking() {
    let err = error(vec![Op::BuildArray(2)], vec![]);
    assert_eq!(err.message, "stack underflow");
    assert_eq!(err.line, 1);
}

#[test]
fn calling_with_too_few_stack_slots_underflows_instead_of_panicking() {
    let err = error(vec![Op::Call(2)], vec![]);
    assert_eq!(err.message, "stack underflow");
    assert_eq!(err.line, 1);
}

#[test]
fn calling_without_a_callee_below_the_arguments_underflows() {
    let err = error(vec![Op::Constant(0), Op::Call(1)], vec![Value::Number(1.0)]);
    assert_eq!(err.message, "stack underflow");
}
