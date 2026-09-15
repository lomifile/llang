use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Constant(usize),
    Null,
    True,
    False,
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    Negate,
    Not,
    Equal,
    Greater,
    Less,
    GetLocal(usize),
    SetLocal(usize),
    DefineGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    Jump(usize),
    JumpIfFalse(usize),
    Loop(usize),
    Call(usize),
    Return,
    Pop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(Rc<str>),
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub code: Vec<Op>,
    pub constants: Vec<Value>,
    pub lines: Vec<u32>,
}
