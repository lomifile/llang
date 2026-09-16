use crate::types::{Chunk, Function, Op, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

fn small_chunk() -> Chunk {
    let mut chunk = Chunk::new();

    let pi = chunk.add_constant(Value::Number(3.5));
    let name = chunk.add_constant(Value::Str(Rc::from("total")));

    chunk.push_op(Op::Constant(pi), 1);
    chunk.push_op(Op::Negate, 1);
    chunk.push_op(Op::DefineGlobal(name), 2);
    chunk.push_op(Op::JumpIfFalse(7), 3);
    chunk.push_op(Op::Pop, 3);
    chunk.push_op(Op::Return, 4);

    chunk
}

#[test]
fn a_chunk_disassembles_every_instruction_in_order() {
    let expected = "\
Disassemble: small
0000    1 Constant 0 -> 3.5
0001    | Negate
0002    2 DefineGlobal 1 -> \"total\"
0003    3 JumpFalse -> 7
0004    | Pop
0005    4 Return
";

    assert_eq!(small_chunk().format_disassembly("small"), expected);
}

#[test]
fn an_empty_chunk_disassembles_to_its_header_alone() {
    assert_eq!(
        Chunk::new().format_disassembly("empty"),
        "Disassemble: empty\n"
    );
}

#[test]
fn a_constant_index_past_the_table_is_reported_rather_than_panicking() {
    let mut chunk = Chunk::new();
    chunk.push_op(Op::Constant(9), 1);

    assert_eq!(
        chunk.format_disassembly("bad"),
        "Disassemble: bad\n0000    1 Constant 9 -> <out of range>\n"
    );
}

#[test]
fn a_value_displays_as_its_source_form() {
    assert_eq!(Value::Null.to_string(), "null");
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(Value::Number(3.5).to_string(), "3.5");
    assert_eq!(Value::Str(Rc::from("total")).to_string(), "total");
}

#[test]
fn a_function_value_displays_with_its_name() {
    let function = Value::Function(Rc::new(Function {
        name: Rc::from("add"),
        arity: 2,
        chunk: Chunk::new(),
    }));

    assert_eq!(function.to_string(), "<fn add>");
}

#[test]
fn an_array_displays_its_elements_in_order() {
    let empty = Value::Array(Rc::new(RefCell::new(Vec::new())));
    assert_eq!(empty.to_string(), "[]");

    let nested = Value::Array(Rc::new(RefCell::new(vec![
        Value::Number(1.0),
        Value::Str(Rc::from("two")),
        Value::Array(Rc::new(RefCell::new(vec![Value::Null]))),
    ])));
    assert_eq!(nested.to_string(), "[1, two, [null]]");
}

#[test]
fn an_object_displays_its_fields_in_key_order() {
    let empty = Value::Object(Rc::new(RefCell::new(HashMap::new())));
    assert_eq!(empty.to_string(), "{}");

    let mut fields = HashMap::new();
    fields.insert("b".to_string(), Value::Number(2.0));
    fields.insert("a".to_string(), Value::Bool(false));
    let object = Value::Object(Rc::new(RefCell::new(fields)));

    assert_eq!(object.to_string(), "{a: false, b: 2}");
}

#[test]
fn a_self_referencing_array_displays_without_recursing_forever() {
    let items = Rc::new(RefCell::new(Vec::new()));
    items.borrow_mut().push(Value::Array(Rc::clone(&items)));

    assert_eq!(Value::Array(items).to_string(), "[[...]]");
}

#[test]
fn a_chunk_disassembles_the_collection_instructions() {
    let mut chunk = Chunk::new();

    let key = chunk.add_constant(Value::Str(Rc::from("name")));
    let array = chunk.add_constant(Value::Array(Rc::new(RefCell::new(vec![Value::Number(
        1.0,
    )]))));

    chunk.push_op(Op::BuildArray(2), 1);
    chunk.push_op(Op::BuildObject(1), 2);
    chunk.push_op(Op::IndexGet, 3);
    chunk.push_op(Op::IndexSet, 4);
    chunk.push_op(Op::MemberGet(key), 5);
    chunk.push_op(Op::MemberSet(key), 6);
    chunk.push_op(Op::Constant(array), 7);

    let expected = "\
Disassemble: collections
0000    1 BuildArray -> 2
0001    2 BuildObject -> 1
0002    3 IndexGet
0003    4 IndexSet
0004    5 MemberGet 0 -> \"name\"
0005    6 MemberSet 0 -> \"name\"
0006    7 Constant 1 -> [1]
";

    assert_eq!(chunk.format_disassembly("collections"), expected);
}

#[test]
fn a_self_referencing_object_displays_without_recursing_forever() {
    let fields = Rc::new(RefCell::new(HashMap::new()));
    fields
        .borrow_mut()
        .insert("self".to_string(), Value::Object(Rc::clone(&fields)));

    assert_eq!(Value::Object(fields).to_string(), "{self: {...}}");
}

#[test]
fn a_value_shared_between_siblings_displays_in_full_each_time() {
    let shared = Rc::new(RefCell::new(vec![Value::Number(1.0)]));
    let outer = Value::Array(Rc::new(RefCell::new(vec![
        Value::Array(Rc::clone(&shared)),
        Value::Array(shared),
    ])));

    assert_eq!(outer.to_string(), "[[1], [1]]");
}

#[test]
fn a_value_borrowed_for_mutation_displays_as_elided() {
    let items = Rc::new(RefCell::new(vec![Value::Number(1.0)]));
    let value = Value::Array(Rc::clone(&items));
    let _borrow = items.borrow_mut();

    assert_eq!(value.to_string(), "[...]");
}
