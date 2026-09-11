use crate::error::TypeError;
use ast::statement::Type;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Binding {
    binding_type: Type,
    mutable: bool,
}

pub struct Environment {
    scopes: Vec<HashMap<String, Binding>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) -> Option<HashMap<String, Binding>> {
        self.scopes.pop()
    }

    pub fn define(&mut self, name: String, typ: Type, mutable: bool) -> Result<(), TypeError> {
        let current = self.scopes.last_mut().expect("global scope always present");

        if current.contains_key(&name) {
            return Err(TypeError {
                message: format!("redefinition of `{name}`"),
            });
        }

        current.insert(
            name,
            Binding {
                binding_type: typ,
                mutable,
            },
        );

        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Some(binding.clone());
            }
        }

        None
    }
}
