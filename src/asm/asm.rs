#![cfg(feature = "asm")]

mod codegen;
mod parser;
mod tokenizer;

use std::{fs::File, io::Read};

use crate::codegen::Codegen;
use crate::codegen::lc3tools_codegen::Lc3ToolsCodegen;
use crate::parser::Parser;
use crate::tokenizer::{Tokenizer, TokenizerErrorInfo, TokenizerErrorKind, TokenizerResult};
use crossterm::style::Stylize;

fn read_entire_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    Ok(buf)
}

fn format_tokenizer_error(err: TokenizerErrorInfo, source: &str) -> String {
    let source_cause: Vec<&str> = source
        .lines()
        .skip(err.line.saturating_sub(4))
        .take(8)
        .collect();

    let cause = source_cause.join("\n");

    format!("Error: {err:?}\n\n{cause}")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let path = &args[1];
    let contents = read_entire_file(path)?;
    println!("{contents}");

    let msg = "Tokenizing".green().bold();
    println!("{msg}");

    let lexer = Tokenizer::new(&contents);
    match lexer.tokenize() {
        TokenizerResult::Ok(tokens) => {
            println!("tokens: {tokens:?}");

            let msg = "Parsing".green().bold();
            println!("{msg}");

            let parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    println!("AST: {ast:#?}");
                    let msg = "Assembling".green().bold();
                    println!("{msg}");

                    let codegen = Lc3ToolsCodegen::new();
                    let res = codegen.generate(ast);
                    let res = String::from_utf8_lossy(&res.bytes);
                    println!("{res}");
                }

                Err(err) => {
                    println!("error: {err:?}");
                }
            }
        }
        TokenizerResult::Err(err) => {
            println!("{}", format_tokenizer_error(err, &contents));
        }

        _ => panic!("Unexpected tokenizer result"),
    }

    Ok(())
}
