#![cfg(feature = "asm")]

mod tokenizer;

use std::{fs::File, io::Read};

use crossterm::style::Stylize;

use crate::tokenizer::Tokenizer;

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
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }


    Ok(())
}
