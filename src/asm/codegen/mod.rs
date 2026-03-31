use crate::parser::Ast;

pub mod lc3tools_codegen;
pub mod partial_instruction;

pub struct CodegenOutput {
    pub bytes: Vec<u8>,
}

pub trait Codegen {
    fn generate(self, ast: Ast) -> CodegenOutput;
}
