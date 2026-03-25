use std::collections::HashMap;

use crate::{codegen::{Codegen, CodegenOutput}, parser::{Ast, AstNode}};

pub struct Lc3ToolsCodegen {
    label_lookup: HashMap<String, u16>,
    generated: Vec<u8>,
}

impl Lc3ToolsCodegen {
    pub fn new() -> Self {
        Self {
            label_lookup: HashMap::new(),
            generated: Vec::new(),
        }
    }

    fn write(&mut self, msg: &str) {
        let b = msg.as_bytes();
        self.generated.extend(b);
    }

    fn generate_header(&mut self) {
        self.write("LC-3 OBJ FILE\n\n");
    }

    fn generate_orig(&mut self, offset: u16, nodes: Vec<AstNode>) {
        self.write(&num_to_4_hexadecimal(offset));
        self.write("\n");

        let short_length = find_code_length_in_shorts(&nodes);
        self.write(&format!("{short_length}\n"));

        // TODO

    }
}

impl Codegen for Lc3ToolsCodegen {
    fn generate(mut self, ast: Ast) -> CodegenOutput {
        self.generate_header();

        self.write(".TEXT\n");

        for orig in ast.orig_sections {
            if let AstNode::Orig(offset, nodes) = orig {

                self.generate_orig(offset, nodes);

            } else {
                panic!("orig_sections did not contain an orig! {orig:?}")
            }
        }
        
        CodegenOutput { bytes: self.generated }
    }
}

fn num_to_4_hexadecimal(num: u16) -> String {
    format!("{num:04X}")
}

fn find_code_length_in_shorts(nodes: &[AstNode]) -> usize {
    let mut length = 0;

    for node in nodes {
        length += node.calculate_byte_length() / 2;
    }

    length
}