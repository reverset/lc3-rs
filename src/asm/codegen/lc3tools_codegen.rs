use core::panic;
use std::collections::HashMap;

use crate::{
    codegen::{Codegen, CodegenOutput},
    parser::{Ast, AstNode},
};

pub struct Lc3ToolsCodegen {
    label_lookup: HashMap<String, usize>,
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

    fn generate_symbol(&mut self) {
        self.write("\n.SYMBOL\n");

        // TODO (may not be implemented)
    }

    fn generate_linker_info(&mut self) {
        self.write("\n.LINKER_INFO\n");

        // TODO
    }

    fn generate_debug(&mut self) {
        self.write("\n.DEBUG\n");
        self.write("# DEBUG SYMBOLS FOR LC3TOOLS\n");

        // TODO
    }

    fn generate_orig(&mut self, offset: u16, nodes: Vec<AstNode>) {
        self.write(&num_to_4_hexadecimal(offset));
        self.write("\n");

        let short_length = find_code_length_in_words(&nodes);
        self.write(&format!("{short_length}\n"));

        let mut word_distance = 0;

        // TODO
        for node in nodes {
            let dist = node.calculate_word_length();
            match node {
                AstNode::Orig(_, _) => panic!("cannot have nested origs."),
                AstNode::Instruction(partial_instruction) => {
                    let instr = partial_instruction
                        .as_u16(offset as usize + word_distance, &self.label_lookup);
                    if let Some(instr) = instr {
                        self.write(&format!("{}\n", num_to_4_hexadecimal(instr)));
                    } else {
                        println!("failed to convert instruction into numeric form: {partial_instruction:?}");
                        panic!("failed to convert instruction into numeric form");
                    }
                }

                AstNode::Label(_) => (), // not handling labels here.
                AstNode::Fill(val) => {
                    self.write(&format!("{}\n", num_to_4_hexadecimal(val as u16)))
                }
                AstNode::Stringz(phrase) => {
                    let bytes = phrase.bytes();

                    for byte in bytes {
                        self.write(&format!("{}\n", num_to_4_hexadecimal(byte as u16)));
                    }
                    self.write("0000\n"); // null terminator
                }
                AstNode::Blkw(size) => {
                    for _ in 0..size {
                        self.write("????\n");
                    }
                }
            }

            word_distance += dist;
        }
    }
}

impl Codegen for Lc3ToolsCodegen {
    fn generate(mut self, ast: Ast) -> CodegenOutput {
        self.label_lookup = ast.scan_for_labels();

        self.generate_header();

        self.write(".TEXT\n");

        for orig in ast.orig_sections {
            if let AstNode::Orig(offset, nodes) = orig {
                self.generate_orig(offset, nodes);
            } else {
                panic!("orig_sections did not contain an orig! {orig:?}")
            }
        }

        self.generate_symbol();
        self.generate_linker_info();
        self.generate_debug();

        CodegenOutput {
            bytes: self.generated,
        }
    }
}

fn num_to_4_hexadecimal(num: u16) -> String {
    format!("{num:04X}")
}

fn find_code_length_in_words(nodes: &[AstNode]) -> usize {
    let mut length = 0;

    for node in nodes {
        length += node.calculate_word_length();
    }

    length
}
