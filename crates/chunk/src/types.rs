use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq)]
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
    And,
    Or,
    BuildArray(usize),
    BuildObject(usize),
    IndexGet,
    IndexSet,
    MemberGet(usize),
    MemberSet(usize),
}

#[derive(Debug)]
pub struct Function {
    pub name: Rc<str>,
    pub arity: usize,
    pub chunk: Chunk,
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(Rc<str>),
    Function(Rc<Function>),
    Array(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b),
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub code: Vec<Op>,
    pub constants: Vec<Value>,
    pub lines: Vec<u32>,
}

thread_local! {
    static VISITING: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

struct CycleGuard;

impl Drop for CycleGuard {
    fn drop(&mut self) {
        VISITING.with_borrow_mut(Vec::pop);
    }
}

fn enter(address: usize) -> Option<CycleGuard> {
    VISITING.with_borrow_mut(|visiting| {
        if visiting.contains(&address) {
            return None;
        }

        visiting.push(address);
        Some(CycleGuard)
    })
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Function(function) => write!(f, "<fn {}>", function.name),
            Value::Array(items) => {
                let Some(_guard) = enter(Rc::as_ptr(items).cast::<()>() as usize) else {
                    return write!(f, "[...]");
                };

                let Ok(items) = items.try_borrow() else {
                    return write!(f, "[...]");
                };

                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Object(fields) => {
                let Some(_guard) = enter(Rc::as_ptr(fields).cast::<()>() as usize) else {
                    return write!(f, "{{...}}");
                };

                let Ok(fields) = fields.try_borrow() else {
                    return write!(f, "{{...}}");
                };

                let mut keys: Vec<&String> = fields.keys().collect();
                keys.sort();

                write!(f, "{{")?;
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {}", fields[*key])?;
                }
                write!(f, "}}")
            }
        }
    }
}
