#![cfg(feature = "asm")]

mod tokenizer;

use std::{fs::File, io::Read};

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

    Ok(())
}
