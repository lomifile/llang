use chunk::types::Chunk;

#[derive(Debug)]
pub struct Compiler {
    pub chunk: Chunk,
    pub scope_depth: usize,
    pub locals: Vec<Local>,
}

#[derive(Debug)]
pub struct Local {
    pub name: String,
    pub depth: usize,
}
