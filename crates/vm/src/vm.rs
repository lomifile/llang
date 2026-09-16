use std::{cell::RefCell, collections::HashMap, rc::Rc};

use chunk::types::{Chunk, Function, Op, Value};

use crate::types::{CallFrame, Flow, RuntimeError, Vm};

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
        }
    }

    pub fn interpret(&mut self, top_level: Chunk) -> Result<(), RuntimeError> {
        let top_fn = Rc::new(Function {
            name: Rc::from("<script>".to_string()),
            arity: 0,
            chunk: top_level,
        });

        self.frames.push(CallFrame {
            function: top_fn,
            ip: 0,
            slot_base: 0,
        });

        self.run()
    }

    pub fn current_frame(&self) -> &CallFrame {
        self.frames
            .last()
            .expect("the vm always runs inside a call frame")
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames
            .last_mut()
            .expect("the vm always runs inside a call frame")
    }

    fn pop_number(&mut self) -> Result<f64, RuntimeError> {
        match self.stack.pop() {
            Some(Value::Number(n)) => Ok(n),
            Some(_) => Err(self.error("expected a number".to_string())),
            None => Err(self.error("stack underflow".to_string())),
        }
    }

    fn pop_boolean(&mut self) -> Result<bool, RuntimeError> {
        match self.stack.pop() {
            Some(Value::Bool(n)) => Ok(n),
            Some(_) => Err(self.error("expected a bool".to_string())),
            None => Err(self.error("stack underflow".to_string())),
        }
    }

    fn pop(&mut self) -> Result<Value, RuntimeError> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => Err(self.error("stack underflow".to_string())),
        }
    }

    fn error(&self, message: String) -> RuntimeError {
        let line = self
            .frames
            .last()
            .and_then(|frame| frame.function.chunk.lines.get(frame.ip.saturating_sub(1)))
            .copied()
            .unwrap_or(0);

        RuntimeError { message, line }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn read_string_constant(&self, i: usize) -> String {
        match &self.current_frame().function.chunk.constants[i] {
            Value::Str(s) => s.to_string(),
            _ => unreachable!(),
        }
    }

    fn next_op(&mut self) -> Option<Op> {
        let frame = self.current_frame();
        let op = *frame.function.chunk.code.get(frame.ip)?;
        self.current_frame_mut().ip += 1;

        Some(op)
    }

    fn binary_number(&mut self, combine: fn(f64, f64) -> f64) -> Result<(), RuntimeError> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.push(Value::Number(combine(a, b)));

        Ok(())
    }

    fn compare_number(&mut self, compare: fn(f64, f64) -> bool) -> Result<(), RuntimeError> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.push(Value::Bool(compare(a, b)));

        Ok(())
    }

    fn binary_boolean(&mut self, combine: fn(bool, bool) -> bool) -> Result<(), RuntimeError> {
        let b = self.pop_boolean()?;
        let a = self.pop_boolean()?;
        self.push(Value::Bool(combine(a, b)));

        Ok(())
    }

    fn divide(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;

        if b == 0.0 {
            return Err(self.error("division by zero".to_string()));
        }

        self.push(Value::Number(a / b));

        Ok(())
    }

    fn equal(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(a == b));

        Ok(())
    }

    fn negate(&mut self) -> Result<(), RuntimeError> {
        let n = self.pop_number()?;
        self.push(Value::Number(-n));

        Ok(())
    }

    fn not(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop_boolean()?;
        self.push(Value::Bool(!b));

        Ok(())
    }

    fn constant(&mut self, i: usize) {
        let value = self.current_frame().function.chunk.constants[i].clone();
        self.push(value);
    }

    fn get_local(&mut self, slot: usize) {
        let index = self.current_frame().slot_base + slot;
        let value = self.stack[index].clone();
        self.push(value);
    }

    fn set_local(&mut self, slot: usize) -> Result<(), RuntimeError> {
        let index = self.current_frame().slot_base + slot;
        let value = self.pop()?;
        self.stack[index] = value;

        Ok(())
    }

    fn define_global(&mut self, i: usize) -> Result<(), RuntimeError> {
        let name = self.read_string_constant(i);
        let value = self.pop()?;
        self.globals.insert(name, value);

        Ok(())
    }

    fn get_global(&mut self, i: usize) -> Result<(), RuntimeError> {
        let name = self.read_string_constant(i);

        match self.globals.get(&name) {
            Some(value) => {
                self.push(value.clone());
                Ok(())
            }
            None => Err(self.error(format!("undefined variable: {name}"))),
        }
    }

    fn set_global(&mut self, i: usize) -> Result<(), RuntimeError> {
        let name = self.read_string_constant(i);
        let value = self.pop()?;

        let Some(slot) = self.globals.get_mut(&name) else {
            return Err(self.error(format!("assignment to undefined variable: {name}")));
        };
        *slot = value;

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn array_index(&self, index: f64) -> Result<usize, RuntimeError> {
        if !index.is_finite() || index < 0.0 || index.fract() != 0.0 {
            return Err(self.error(format!("invalid array index: {index}")));
        }

        Ok(index as usize)
    }

    fn build_array(&mut self, count: usize) -> Result<(), RuntimeError> {
        let Some(start) = self.stack.len().checked_sub(count) else {
            return Err(self.error("stack underflow".to_string()));
        };

        let elements = self.stack.drain(start..).collect();
        self.push(Value::Array(Rc::new(RefCell::new(elements))));

        Ok(())
    }

    fn jump_if_false(&mut self, target: usize) -> Result<(), RuntimeError> {
        if !self.pop_boolean()? {
            self.current_frame_mut().ip = target;
        }

        Ok(())
    }

    fn call(&mut self, argc: usize) -> Result<(), RuntimeError> {
        let Some(fn_index) = self
            .stack
            .len()
            .checked_sub(argc)
            .and_then(|base| base.checked_sub(1))
        else {
            return Err(self.error("stack underflow".to_string()));
        };

        let callee = self.stack[fn_index].clone();

        let Value::Function(function) = callee else {
            return Err(self.error("can only call function".to_string()));
        };

        if argc != function.arity {
            return Err(self.error("wrong argument count".to_string()));
        }

        let slot_base = fn_index + 1;
        self.frames.push(CallFrame {
            function,
            ip: 0,
            slot_base,
        });

        Ok(())
    }

    fn ret(&mut self) -> Result<Flow, RuntimeError> {
        let return_value = self.pop()?;

        let Some(finished) = self.frames.pop() else {
            return Err(self.error("call frame doesn't exist".to_string()));
        };

        if self.frames.is_empty() {
            self.stack.clear();
            return Ok(Flow::Halt);
        }

        self.stack.truncate(finished.slot_base - 1);
        self.push(return_value);

        Ok(Flow::Continue)
    }

    fn step(&mut self, op: Op) -> Result<Flow, RuntimeError> {
        match op {
            Op::Constant(i) => self.constant(i),
            Op::Null => self.push(Value::Null),
            Op::True => self.push(Value::Bool(true)),
            Op::False => self.push(Value::Bool(false)),
            Op::Add => self.binary_number(|a, b| a + b)?,
            Op::Sub => self.binary_number(|a, b| a - b)?,
            Op::Mul => self.binary_number(|a, b| a * b)?,
            Op::Mod => self.binary_number(|a, b| a % b)?,
            Op::Div => self.divide()?,
            Op::Negate => self.negate()?,
            Op::Not => self.not()?,
            Op::Equal => self.equal()?,
            Op::Greater => self.compare_number(|a, b| a > b)?,
            Op::Less => self.compare_number(|a, b| a < b)?,
            Op::And => self.binary_boolean(|a, b| a && b)?,
            Op::Or => self.binary_boolean(|a, b| a || b)?,
            Op::Pop => {
                self.pop()?;
            }
            Op::GetLocal(slot) => self.get_local(slot),
            Op::SetLocal(slot) => self.set_local(slot)?,
            Op::DefineGlobal(i) => self.define_global(i)?,
            Op::GetGlobal(i) => self.get_global(i)?,
            Op::SetGlobal(i) => self.set_global(i)?,
            Op::Jump(target) | Op::Loop(target) => self.current_frame_mut().ip = target,
            Op::JumpIfFalse(target) => self.jump_if_false(target)?,
            Op::Call(argc) => self.call(argc)?,
            Op::Return => return self.ret(),
            Op::BuildArray(count) => self.build_array(count)?,
            Op::BuildObject(count) => {
                let mut map = HashMap::new();
                for _ in 0..count {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    let Value::Str(key_string) = key else {
                        return Err(self.error("cannot find key".to_string()));
                    };
                    map.insert(key_string.to_string(), value);
                }
                self.push(Value::Object(Rc::new(RefCell::new(map))));
            }
            Op::IndexGet => {
                let index = self.pop_number()?;
                let array_value = self.pop()?;
                let Value::Array(array) = array_value else {
                    return Err(self.error("not an array".to_string()));
                };

                let i = self.array_index(index)?;
                let borrowed = array.borrow();
                let result = borrowed.get(i).cloned().unwrap_or(Value::Null);
                self.push(result);
            }
            Op::IndexSet => {
                let value = self.pop()?;
                let index = self.pop_number()?;
                let array_value = self.pop()?;
                let Value::Array(array) = array_value else {
                    return Err(self.error("not an array".to_string()));
                };
                let i = self.array_index(index)?;
                let mut mut_borrow = array.borrow_mut();
                if i < mut_borrow.len() {
                    mut_borrow[i] = value;
                } else {
                    return Err(self.error("array out of bounds".to_string()));
                }
            }
            Op::MemberGet(field_const) => {
                let field = self.read_string_constant(field_const);
                let object_val = self.pop()?;
                let Value::Object(object) = object_val else {
                    return Err(self.error("not an object".to_string()));
                };
                let borrowed = object.borrow();
                let result = borrowed.get(&field).cloned().unwrap_or(Value::Null);
                self.push(result);
            }
            Op::MemberSet(field_const) => {
                let field = self.read_string_constant(field_const);
                let value = self.pop()?;
                let object_val = self.pop()?;
                let Value::Object(object) = object_val else {
                    return Err(self.error("not an object".to_string()));
                };
                object.borrow_mut().insert(field, value);
            }
        }

        Ok(Flow::Continue)
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        while let Some(op) = self.next_op() {
            if self.step(op)? == Flow::Halt {
                break;
            }
        }

        Ok(())
    }
}
