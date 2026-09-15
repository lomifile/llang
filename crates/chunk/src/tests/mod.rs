use crate::types::{Chunk, Op, Value};
use std::rc::Rc;

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
