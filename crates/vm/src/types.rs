use std::{collections::HashMap, fmt, rc::Rc};

use chunk::types::{Function, Value};

#[derive(Debug)]
pub struct CallFrame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub slot_base: usize,
}

#[derive(Debug)]
pub struct Vm {
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub frames: Vec<CallFrame>,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Halt,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.line, self.message)
    }
}

impl std::error::Error for RuntimeError {}
