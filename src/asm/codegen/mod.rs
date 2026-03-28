use crate::parser::Ast;

pub mod lc3tools_codegen;

pub struct CodegenOutput {
    pub bytes: Vec<u8>,
}

pub trait Codegen {
    fn generate(self, ast: Ast) -> CodegenOutput;
}
