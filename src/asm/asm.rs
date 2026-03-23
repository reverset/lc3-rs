#![cfg(feature = "asm")]

mod parser;
mod tokenizer;

use std::{fs::File, io::Read};

use crate::parser::Parser;
use crate::tokenizer::Tokenizer;
use crossterm::style::Stylize;

fn read_entire_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    Ok(buf)
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
        Ok(tokens) => {
            println!("tokens: {tokens:?}");

            let msg = "Parsing".green().bold();
            println!("{msg}");

            let parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    println!("AST: {ast:#?}");
                }

                Err(err) => {
                    println!("error: {err:?}");
                }
            }
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }

    Ok(())
}
