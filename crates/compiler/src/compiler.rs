use std::rc::Rc;

use ast::statement::{
    Declaration, DeclarationKind, Expression, ExpressionKind, ForInit, ForStep, Statement,
    StatementKind,
};
use chunk::types::{Chunk, Function, Op, Value};
use token::keywords::Operator;

use crate::types::{Compiler, Local};

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, line: u32) {
        self.scope_depth -= 1;

        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth {
                break;
            }
            self.emit(Op::Pop, line);
            self.locals.pop();
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].name == name {
                return Some(i);
            }
        }
        None
    }

    fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
    }

    fn emit(&mut self, op: Op, line: u32) -> usize {
        self.chunk.push_op(op, line)
    }

    fn emit_binary_op(&mut self, op: Operator, line: u32) {
        match op {
            Operator::Plus => {
                self.emit(Op::Add, line);
            }
            Operator::Minus => {
                self.emit(Op::Sub, line);
            }
            Operator::Multiply => {
                self.emit(Op::Mul, line);
            }
            Operator::Divide => {
                self.emit(Op::Div, line);
            }
            Operator::Modulo => {
                self.emit(Op::Mod, line);
            }
            Operator::GreaterThan => {
                self.emit(Op::Greater, line);
            }
            Operator::LessThan => {
                self.emit(Op::Less, line);
            }
            Operator::Eq => {
                self.emit(Op::Equal, line);
            }
            Operator::And => {
                self.emit(Op::And, line);
            }
            Operator::Or => {
                self.emit(Op::Or, line);
            }
            Operator::GreaterOrEqual => {
                self.emit(Op::Less, line);
                self.emit(Op::Not, line);
            }
            Operator::LessOrEqual => {
                self.emit(Op::Greater, line);
                self.emit(Op::Not, line);
            }
            Operator::NotEq => {
                self.emit(Op::Equal, line);
                self.emit(Op::Not, line);
            }
            _ => unreachable!(),
        }
    }

    fn compile_assign(&mut self, target: &Expression, value: &Expression, line: u32) {
        match &target.kind {
            ExpressionKind::Identifier(name) => {
                self.compile_expression(value.clone());
                match self.resolve_local(name) {
                    Some(slot) => {
                        self.emit(Op::SetLocal(slot), line);
                    }
                    None => {
                        let i = self.chunk.add_constant(Value::Str(Rc::from(name.clone())));
                        self.emit(Op::SetGlobal(i), line);
                    }
                }
            }
            ExpressionKind::Index { target, index } => {
                self.compile_expression(*target.clone());
                self.compile_expression(*index.clone());
                self.compile_expression(value.clone());
                self.emit(Op::IndexSet, line);
            }
            ExpressionKind::Member { target, field } => {
                self.compile_expression(*target.clone());
                let field_const = self.chunk.add_constant(Value::Str(Rc::from(field.clone())));
                self.compile_expression(value.clone());
                self.emit(Op::MemberSet(field_const), line);
            }
            _ => unreachable!(),
        }
    }

    fn compile_for_step(&mut self, step: ForStep, line: u32) {
        match step {
            ForStep::Assign { target, value } => self.compile_assign(&target, &value, line),
            ForStep::Increment(target) => self.compile_incdec(&target, Op::Add, line),
            ForStep::Decrement(target) => self.compile_incdec(&target, Op::Sub, line),
        }
    }

    fn compile_incdec(&mut self, target: &Expression, op: Op, line: u32) {
        match &target.kind {
            ExpressionKind::Identifier(name) => match self.resolve_local(name) {
                Some(slot) => {
                    self.emit(Op::GetLocal(slot), line);
                    let one = self.chunk.add_constant(Value::Number(1.0));
                    self.emit(Op::Constant(one), line);
                    self.emit(op, line);
                    self.emit(Op::SetLocal(slot), line);
                }
                None => {
                    let i = self.chunk.add_constant(Value::Str(Rc::from(name.as_str())));
                    self.emit(Op::GetGlobal(i), line);
                    let one = self.chunk.add_constant(Value::Number(1.0));
                    self.emit(Op::Constant(one), line);
                    self.emit(op, line);
                    self.emit(Op::SetGlobal(i), line);
                }
            },
            _ => unreachable!(),
        }
    }

    pub(crate) fn compile_declaration(&mut self, declaration: Declaration) {
        let line = declaration.span.line;
        match declaration.kind {
            DeclarationKind::Statement(statement) => {
                self.compile_statement(statement);
            }
            DeclarationKind::Let { name, ty: _, init } => {
                self.compile_expression(init);
                if self.scope_depth == 0 {
                    let i = self.chunk.add_constant(Value::Str(Rc::from(name)));
                    self.emit(Op::DefineGlobal(i), line);
                } else {
                    self.add_local(name);
                }
            }
            DeclarationKind::Const { name, ty: _, init } => {
                self.compile_expression(init);
                if self.scope_depth == 0 {
                    let i = self.chunk.add_constant(Value::Str(Rc::from(name)));
                    self.emit(Op::DefineGlobal(i), line);
                } else {
                    self.add_local(name);
                }
            }
            DeclarationKind::Function {
                name,
                params,
                return_type,
                body,
            } => {
                let mut sub = Compiler::new();
                sub.begin_scope();
                let arity = params.len();
                for param in params {
                    sub.add_local(param.name);
                }

                sub.compile_statement(*body);
                sub.emit(Op::Null, line);
                sub.emit(Op::Return, line);

                let fn_chunk = sub.chunk;
                let fn_value = Value::Function(Rc::new(Function {
                    name: Rc::from(name.as_str()),
                    arity,
                    chunk: fn_chunk,
                }));

                let i = self.chunk.add_constant(fn_value);
                self.emit(Op::Constant(i), line);
                let name_i = self.chunk.add_constant(Value::Str(Rc::from(name)));
                self.emit(Op::DefineGlobal(name_i), line);
            }
        }
    }

    pub(crate) fn compile_statement(&mut self, statement: Statement) {
        let line = statement.span.line;
        match statement.kind {
            StatementKind::ExpressionStatement(expression) => {
                self.compile_expression(expression);
                self.emit(Op::Pop, line);
            }
            StatementKind::Return(maybe_expression) => {
                match maybe_expression {
                    Some(expr) => self.compile_expression(expr),
                    None => {
                        self.emit(Op::Null, line);
                    }
                }
                self.emit(Op::Return, line);
            }
            StatementKind::Assign { target, value } => self.compile_assign(&target, &value, line),
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expression(condition);

                match else_branch {
                    None => {
                        let jump_index = self.emit(Op::JumpIfFalse(0), line);
                        self.compile_statement(*then_branch);
                        self.chunk.patch_jump(jump_index, self.chunk.code_len());
                    }
                    Some(else_stmt) => {
                        let else_jump = self.emit(Op::JumpIfFalse(0), line);
                        self.compile_statement(*then_branch);
                        let end_jump = self.emit(Op::Jump(0), line);
                        self.chunk.patch_jump(else_jump, self.chunk.code_len());
                        self.compile_statement(*else_stmt);
                        self.chunk.patch_jump(end_jump, self.chunk.code_len());
                    }
                }
            }
            StatementKind::While { condition, body } => {
                let loop_start = self.chunk.code_len();
                self.compile_expression(condition);
                let exit_jump = self.emit(Op::JumpIfFalse(0), line);
                self.compile_statement(*body);
                self.emit(Op::Loop(loop_start), line);
                self.chunk.patch_jump(exit_jump, self.chunk.code_len());
            }
            StatementKind::For {
                init,
                condition,
                step,
                body,
            } => {
                self.begin_scope();

                match init {
                    ForInit::Let(declaration) => self.compile_declaration(*declaration),
                    ForInit::Step(step) => self.compile_for_step(step, line),
                    ForInit::None => {}
                }

                let loop_start = self.chunk.code_len();

                let exit_jump = match condition {
                    Some(cond) => {
                        self.compile_expression(cond);
                        Some(self.emit(Op::JumpIfFalse(0), line))
                    }
                    None => None,
                };

                self.compile_statement(*body);

                match step {
                    Some(s) => self.compile_for_step(s, line),
                    None => {}
                }

                self.emit(Op::Loop(loop_start), line);

                if let Some(idx) = exit_jump {
                    self.chunk.patch_jump(idx, self.chunk.code_len());
                }

                self.end_scope(line);
            }
            StatementKind::Block(declarations) => {
                self.begin_scope();
                for declaration in declarations {
                    self.compile_declaration(declaration);
                }
                self.end_scope(line);
            }
            StatementKind::Increment(expression) => self.compile_incdec(&expression, Op::Add, line),
            StatementKind::Decrement(expression) => self.compile_incdec(&expression, Op::Sub, line),
        }
    }

    pub(crate) fn compile_expression(&mut self, expression: Expression) {
        let line = expression.span.line;

        match expression.kind {
            ExpressionKind::LiteralNumber(n) => {
                let i = self.chunk.add_constant(Value::Number(n));
                self.emit(Op::Constant(i), line);
            }
            ExpressionKind::LiteralString(s) => {
                let i = self.chunk.add_constant(Value::Str(Rc::from(s)));
                self.emit(Op::Constant(i), line);
            }
            ExpressionKind::LiteralBoolean(true) => {
                self.emit(Op::True, line);
            }
            ExpressionKind::LiteralBoolean(false) => {
                self.emit(Op::False, line);
            }
            ExpressionKind::LiteralNull => {
                self.emit(Op::Null, line);
            }
            ExpressionKind::Identifier(identifier) => match self.resolve_local(&identifier) {
                Some(slot) => {
                    self.emit(Op::GetLocal(slot), line);
                }
                None => {
                    let i = self.chunk.add_constant(Value::Str(Rc::from(identifier)));
                    self.emit(Op::GetGlobal(i), line);
                }
            },
            ExpressionKind::BinaryOp { op, left, right } => {
                self.compile_expression(*left);
                self.compile_expression(*right);
                self.emit_binary_op(op, line);
            }
            ExpressionKind::UnaryOp { op, operand } => {
                self.compile_expression(*operand);
                match op {
                    Operator::Minus => self.emit(Op::Negate, line),
                    Operator::Not => self.emit(Op::Not, line),
                    _ => unreachable!(),
                };
            }
            ExpressionKind::Call { callee, args } => {
                let name_i = self.chunk.add_constant(Value::Str(Rc::from(callee)));
                let argc = args.len();
                self.emit(Op::GetGlobal(name_i), line);
                for arg in args {
                    self.compile_expression(arg);
                }
                self.emit(Op::Call(argc), line);
            }
            ExpressionKind::ArrayLiteral(elements) => {
                let count = elements.len();
                for element in elements {
                    self.compile_expression(element);
                }

                self.emit(Op::BuildArray(count), line);
            }
            ExpressionKind::ObjectLiteral(pairs) => {
                let count = pairs.len();
                for (key, value) in pairs {
                    let key_const = self.chunk.add_constant(Value::Str(Rc::from(key)));
                    self.emit(Op::Constant(key_const), line);
                    self.compile_expression(value);
                }
                self.emit(Op::BuildObject(count), line);
            }
            ExpressionKind::Index { target, index } => {
                self.compile_expression(*target);
                self.compile_expression(*index);
                self.emit(Op::IndexGet, line);
            }
            ExpressionKind::Member { target, field } => {
                self.compile_expression(*target);
                let field_const = self.chunk.add_constant(Value::Str(Rc::from(field)));
                self.emit(Op::MemberGet(field_const), line);
            }
        }
    }
}
