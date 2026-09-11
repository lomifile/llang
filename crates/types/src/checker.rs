use crate::{environment::Environment, error::TypeError};
use ast::statement::{Declaration, Expression, ExpressionKind, Statement};

pub struct Checker {
    env: Environment,
}

impl Checker {
    pub fn new() -> Self {
        Checker {
            env: Environment::new(),
        }
    }

    fn type_of_expression(&self, expression: &Expression) -> Result<Type, TypeError> {}

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), TypeError> {
        todo!()
    }

    fn check_declaration(&mut self, declaration: &Declaration) -> Result<(), TypeError> {
        todo!()
    }
}
