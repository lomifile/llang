use crate::types::{Chunk, Op, Value};

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn push_op(&mut self, op: Op, line: u32) -> usize {
        self.code.push(op);
        self.lines.push(line);

        self.code.len() - 1
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        print!("{}", self.format_disassembly(name));
    }

    pub fn format_disassembly(&self, name: &str) -> String {
        let mut out = format!("Disassemble: {name}\n");

        for index in 0..self.code.len() {
            out.push_str(&self.format_instruction(index));
            out.push('\n');
        }

        out
    }

    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    pub fn patch_jump(&mut self, index: usize, target: usize) {
        match self.code[index] {
            Op::Jump(_) => self.code[index] = Op::Jump(target),
            Op::JumpIfFalse(_) => self.code[index] = Op::JumpIfFalse(target),
            Op::Loop(_) => self.code[index] = Op::Loop(target),
            _ => unreachable!(),
        }
    }

    fn constant(&self, index: usize) -> String {
        match self.constants.get(index) {
            Some(Value::Str(s)) => format!("\"{s}\""),
            Some(value) => value.to_string(),
            None => "<out of range>".to_string(),
        }
    }

    fn format_instruction(&self, index: usize) -> String {
        let line = if index > 0 && self.lines.get(index) == self.lines.get(index - 1) {
            "   | ".to_string()
        } else {
            match self.lines.get(index) {
                Some(line) => format!("{line:4} "),
                None => "   ? ".to_string(),
            }
        };

        let op = match self.code[index] {
            Op::Constant(c) => format!("Constant {} -> {}", c, self.constant(c)),
            Op::Null => "Null".to_string(),
            Op::True => "True".to_string(),
            Op::False => "False".to_string(),
            Op::Add => "Add".to_string(),
            Op::Sub => "Sub".to_string(),
            Op::Mul => "Mul".to_string(),
            Op::Mod => "Mod".to_string(),
            Op::Div => "Div".to_string(),
            Op::Negate => "Negate".to_string(),
            Op::Not => "Not".to_string(),
            Op::Equal => "Equal".to_string(),
            Op::Greater => "Greater".to_string(),
            Op::Less => "Less".to_string(),
            Op::GetLocal(s) => format!("GetLocal -> {s}"),
            Op::SetLocal(s) => format!("SetLocal -> {s}"),
            Op::DefineGlobal(g) => format!("DefineGlobal {} -> {}", g, self.constant(g)),
            Op::GetGlobal(g) => format!("GetGlobal {} -> {}", g, self.constant(g)),
            Op::SetGlobal(g) => format!("SetGlobal {} -> {}", g, self.constant(g)),
            Op::Jump(j) => format!("Jump -> {j}"),
            Op::JumpIfFalse(j) => format!("JumpFalse -> {j}"),
            Op::Loop(l) => format!("Loop -> {l}"),
            Op::Call(c) => format!("Call -> {c}"),
            Op::Return => "Return".to_string(),
            Op::Pop => "Pop".to_string(),
            Op::And => "And".to_string(),
            Op::Or => "Or".to_string(),
            Op::BuildArray(n) => format!("BuildArray -> {n}"),
            Op::BuildObject(n) => format!("BuildObject -> {n}"),
            Op::IndexGet => "IndexGet".to_string(),
            Op::IndexSet => "IndexSet".to_string(),
            Op::MemberGet(m) => format!("MemberGet {} -> {}", m, self.constant(m)),
            Op::MemberSet(m) => format!("MemberSet {} -> {}", m, self.constant(m)),
        };

        format!("{index:04} {line}{op}")
    }
}
