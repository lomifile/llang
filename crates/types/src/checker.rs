use std::{collections::HashMap, iter::zip};

use crate::{environment::Environment, error::TypeError, functions::FunctionSignature};
use ast::statement::{
    Declaration, DeclarationKind, Expression, ExpressionKind, ForInit, ForStep, FunctionParam,
    Statement, StatementKind, Type,
};
use token::keywords::Operator;

#[derive(Debug)]
pub struct Checker {
    env: Environment,
    functions: HashMap<String, FunctionSignature>,
    current_return_type: Option<Type>,
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker {
    pub fn new() -> Self {
        Checker {
            env: Environment::new(),
            functions: HashMap::new(),
            current_return_type: None,
        }
    }

    pub fn check_program(&mut self, declarations: &[Declaration]) -> Result<(), TypeError> {
        for declaration in declarations {
            if let DeclarationKind::Function {
                name,
                params,
                return_type,
                ..
            } = &declaration.kind
            {
                let signatrue = FunctionSignature {
                    params: params.iter().map(|p| p.param_type.clone()).collect(),
                    return_type: return_type.clone(),
                };

                if self.functions.contains_key(name) {
                    return Err(TypeError {
                        message: format!("redefinitoin of function `{name}`"),
                        span: declaration.span,
                    });
                }

                self.functions.insert(name.clone(), signatrue);
            }
        }

        for declaration in declarations {
            self.check_declaration(declaration)?;
        }

        Ok(())
    }

    fn type_of_binary_expression(
        &self,
        expression: &Expression,
        op: Operator,
        left: &Expression,
        right: &Expression,
    ) -> Result<Type, TypeError> {
        let lt = self.type_of_expression(left)?;
        let rt = self.type_of_expression(right)?;

        match op {
            Operator::Plus
            | Operator::Minus
            | Operator::Multiply
            | Operator::Divide
            | Operator::Modulo => {
                if lt == Type::Number && rt == Type::Number {
                    Ok(Type::Number)
                } else {
                    Err(TypeError {
                        message: "operator + | - | / | * should be numbers".to_string(),
                        span: expression.span,
                    })
                }
            }

            Operator::LessThan
            | Operator::GreaterThan
            | Operator::LessOrEqual
            | Operator::GreaterOrEqual => {
                if lt == Type::Number && rt == Type::Number {
                    Ok(Type::Boolean)
                } else {
                    Err(TypeError {
                        message: "operator >= <= == > < should be boolean expressions".to_string(),
                        span: expression.span,
                    })
                }
            }

            Operator::Eq | Operator::NotEq => {
                if lt == rt {
                    Ok(Type::Boolean)
                } else {
                    Err(TypeError {
                        message: "operators = and != should be boolean".to_string(),
                        span: expression.span,
                    })
                }
            }

            Operator::And | Operator::Or => {
                if lt == Type::Boolean && rt == Type::Boolean {
                    Ok(Type::Boolean)
                } else {
                    Err(TypeError {
                        message: "&& and || should be boolean operators".to_string(),
                        span: expression.span,
                    })
                }
            }

            _ => unreachable!(),
        }
    }

    fn type_of_unary_expression(
        &self,
        expression: &Expression,
        op: Operator,
        operand: &Expression,
    ) -> Result<Type, TypeError> {
        let ot = self.type_of_expression(operand)?;

        match op {
            Operator::Not => {
                if ot == Type::Boolean {
                    Ok(Type::Boolean)
                } else {
                    Err(TypeError {
                        message: "not operator should be boolean".to_string(),
                        span: expression.span,
                    })
                }
            }
            Operator::Minus => {
                if ot == Type::Number {
                    Ok(Type::Number)
                } else {
                    Err(TypeError {
                        message: "- should be number".to_string(),
                        span: expression.span,
                    })
                }
            }

            _ => unreachable!(),
        }
    }

    fn type_of_call(
        &self,
        expression: &Expression,
        callee: &str,
        args: &[Expression],
    ) -> Result<Type, TypeError> {
        let Some(signature) = self.functions.get(callee) else {
            return Err(TypeError {
                message: format!("unknown function `{callee}`"),
                span: expression.span,
            });
        };
        let signature = signature.clone();

        if args.len() != signature.params.len() {
            let n = args.len();
            let m = signature.params.len();
            return Err(TypeError {
                message: format!("format `{callee}` expects {m}, got {n}"),
                span: expression.span,
            });
        }

        for (arg, param_type) in zip(args, signature.params) {
            let arg_type = self.type_of_expression(arg)?;
            if arg_type != param_type {
                return Err(TypeError {
                    message: format!(
                        "argument type mismatch: expected {param_type}, got {arg_type}"
                    ),
                    span: arg.span,
                });
            }
        }

        Ok(signature.return_type)
    }

    fn type_of_array_literal(
        &self,
        expression: &Expression,
        elements: &[Expression],
    ) -> Result<Type, TypeError> {
        let Some(first) = elements.first() else {
            return Err(TypeError {
                message: "cannot infer type of empty array".to_string(),
                span: expression.span,
            });
        };

        let first_type = self.type_of_expression(first)?;

        for element in &elements[1..] {
            let element_type = self.type_of_expression(element)?;
            if element_type != first_type {
                return Err(TypeError {
                    message: format!(
                        "array elements must all be the same type: expected {first_type}, got {element_type}"
                    ),
                    span: element.span,
                });
            }
        }

        Ok(Type::Array(Box::new(first_type)))
    }

    fn type_of_index(
        &self,
        target: &Expression,
        index: &Expression,
    ) -> Result<Type, TypeError> {
        let target_type = self.type_of_expression(target)?;
        let index_type = self.type_of_expression(index)?;

        if index_type != Type::Number {
            return Err(TypeError {
                message: format!("array index must be Number, got {index_type}"),
                span: index.span,
            });
        }

        match &target_type {
            Type::Array(element_type) => Ok((**element_type).clone()),
            _ => Err(TypeError {
                message: format!("cannot index into {target_type}"),
                span: target.span,
            }),
        }
    }

    fn type_of_expression(&self, expression: &Expression) -> Result<Type, TypeError> {
        match &expression.kind {
            ExpressionKind::LiteralNumber(_) => Ok(Type::Number),
            ExpressionKind::LiteralString(_) => Ok(Type::String),
            ExpressionKind::LiteralBoolean(_) => Ok(Type::Boolean),
            ExpressionKind::LiteralNull => Ok(Type::Null),
            ExpressionKind::Identifier(name) => match self.env.lookup(name) {
                Some(bind) => Ok(bind.binding_type),
                None => Err(TypeError {
                    message: format!("unknown variable: {name}"),
                    span: expression.span,
                }),
            },
            ExpressionKind::BinaryOp { op, left, right } => {
                self.type_of_binary_expression(expression, *op, left, right)
            }
            ExpressionKind::UnaryOp { op, operand } => {
                self.type_of_unary_expression(expression, *op, operand)
            }
            ExpressionKind::Call { callee, args } => self.type_of_call(expression, callee, args),
            ExpressionKind::ArrayLiteral(elements) => {
                self.type_of_array_literal(expression, elements)
            }
            ExpressionKind::ObjectLiteral(pairs) => {
                for (_, value) in pairs {
                    self.type_of_expression(value)?;
                }
                Ok(Type::Object)
            }
            ExpressionKind::Index { target, index } => self.type_of_index(target, index),
            ExpressionKind::Member { target, .. } => {
                self.type_of_expression(target)?;
                Ok(Type::Object)
            }
        }
    }

    fn check_assignment(&self, target: &Expression, value: &Expression) -> Result<(), TypeError> {
        let target_type = self.type_of_expression(target)?;
        let value_type = self.type_of_expression(value)?;

        if target_type != value_type {
            return Err(TypeError {
                message: format!("assignment mismatch: expected {target_type}, got {value_type}"),
                span: value.span,
            });
        }

        match &target.kind {
            ExpressionKind::Identifier(name) => {
                let binding = self
                    .env
                    .lookup(name.as_str())
                    .expect("assignment target resolved during synthesis");
                if binding.mutable {
                    Ok(())
                } else {
                    Err(TypeError {
                        message: format!("cannot assign to const {name}"),
                        span: target.span,
                    })
                }
            }
            ExpressionKind::Index { .. } | ExpressionKind::Member { .. } => Ok(()),
            _ => Err(TypeError {
                message: "invalid assignment target".to_string(),
                span: target.span,
            }),
        }
    }

    fn check_for_step(&self, step: &ForStep) -> Result<(), TypeError> {
        match step {
            ForStep::Assign { target, value } => self.check_assignment(target, value),
            ForStep::Increment(target) | ForStep::Decrement(target) => {
                self.check_incdec_target(target)
            }
        }
    }

    fn check_incdec_target(&self, target: &Expression) -> Result<(), TypeError> {
        let target_type = self.type_of_expression(target)?;

        if target_type != Type::Number {
            return Err(TypeError {
                message: format!("++ and -- require a Number, got {target_type}"),
                span: target.span,
            });
        }

        Ok(())
    }

    fn check_return(
        &self,
        stmt: &Statement,
        maybe_expression: Option<&Expression>,
    ) -> Result<(), TypeError> {
        let Some(expected) = self.current_return_type.clone() else {
            return Err(TypeError {
                message: "return after function".to_string(),
                span: stmt.span,
            });
        };

        let Some(expression) = maybe_expression else {
            if expected != Type::Void {
                return Err(TypeError {
                    message: format!("function  must return {expected}"),
                    span: stmt.span,
                });
            }

            return Ok(());
        };

        let actual = self.type_of_expression(expression)?;

        if expected == Type::Void {
            return Err(TypeError {
                message: "void function cannot return a value".to_string(),
                span: expression.span,
            });
        }

        if actual != expected {
            return Err(TypeError {
                message: format!("return type mismatch: expected {expected}, got: {actual}"),
                span: expression.span,
            });
        }

        Ok(())
    }

    fn check_for(
        &mut self,
        init: &ForInit,
        condition: Option<&Expression>,
        step: Option<&ForStep>,
        body: &Statement,
    ) -> Result<(), TypeError> {
        self.env.push_scope();

        let checked = self.check_for_parts(init, condition, step, body);
        self.env.pop_scope();

        checked
    }

    fn check_for_parts(
        &mut self,
        init: &ForInit,
        condition: Option<&Expression>,
        step: Option<&ForStep>,
        body: &Statement,
    ) -> Result<(), TypeError> {
        match init {
            ForInit::Let(declaration) => self.check_declaration(declaration)?,
            ForInit::Step(step) => self.check_for_step(step)?,
            ForInit::None => {}
        }

        if let Some(cond) = condition {
            let condition_type = self.type_of_expression(cond)?;
            if condition_type != Type::Boolean {
                return Err(TypeError {
                    message: format!("for condition must be Boolean, got {condition_type}"),
                    span: cond.span,
                });
            }
        }

        if let Some(s) = step {
            self.check_for_step(s)?;
        }

        self.check_statement(body)
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), TypeError> {
        match &stmt.kind {
            StatementKind::ExpressionStatement(expression) => {
                self.type_of_expression(expression)?;
                Ok(())
            }

            StatementKind::Increment(e) | StatementKind::Decrement(e) => {
                self.check_incdec_target(e)
            }

            StatementKind::For {
                init,
                condition,
                step,
                body,
            } => self.check_for(init, condition.as_ref(), step.as_ref(), body),

            StatementKind::Assign { target, value } => self.check_assignment(target, value),

            StatementKind::Return(maybe_expression) => {
                self.check_return(stmt, maybe_expression.as_ref())
            }

            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.type_of_expression(condition)?;
                if condition_type != Type::Boolean {
                    return Err(TypeError {
                        message: format!("if condition must be Boolean, got {condition_type}"),
                        span: condition.span,
                    });
                }

                self.check_statement(then_branch)?;

                if let Some(branch) = else_branch {
                    self.check_statement(branch)?;
                }

                Ok(())
            }

            StatementKind::While { condition, body } => {
                let condition_type = self.type_of_expression(condition)?;
                if condition_type != Type::Boolean {
                    return Err(TypeError {
                        message: format!("while condition must be boolean, got {condition_type}"),
                        span: condition.span,
                    });
                }
                self.check_statement(body)
            }

            StatementKind::Block(declarations) => {
                self.env.push_scope();

                let checked = declarations
                    .iter()
                    .try_for_each(|declaration| self.check_declaration(declaration));

                self.env.pop_scope();

                checked
            }
        }
    }

    fn check_function_body(
        &mut self,
        declaration: &Declaration,
        params: &[FunctionParam],
        body: &Statement,
    ) -> Result<(), TypeError> {
        for param in params {
            self.env.define(
                param.name.clone(),
                param.param_type.clone(),
                true,
                declaration.span,
            )?;
        }

        self.check_statement(body)
    }

    fn check_declaration(&mut self, declaration: &Declaration) -> Result<(), TypeError> {
        match &declaration.kind {
            DeclarationKind::Let { name, ty, init } => {
                let init_type = self.type_of_expression(init)?;
                if init_type != *ty {
                    return Err(TypeError {
                        message: format!("type mismatch: expected {ty}, got {init_type}"),
                        span: init.span,
                    });
                }
                self.env.define(name.clone(), ty.clone(), true, init.span)
            }

            DeclarationKind::Const { name, ty, init } => {
                let init_type = self.type_of_expression(init)?;
                if init_type != *ty {
                    return Err(TypeError {
                        message: format!("type mismatch: expected {ty}, got {init_type}"),
                        span: init.span,
                    });
                }
                self.env.define(name.clone(), ty.clone(), false, init.span)
            }

            DeclarationKind::Function {
                params,
                return_type,
                body,
                ..
            } => {
                let save = self.current_return_type.replace(return_type.clone());

                self.env.push_scope();
                let checked = self.check_function_body(declaration, params, body);
                self.env.pop_scope();

                self.current_return_type = save;

                checked
            }

            DeclarationKind::Statement(stmt) => self.check_statement(stmt),
        }
    }
}
