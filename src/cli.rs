#![cfg(feature = "cli")]

mod asm;
mod cli_tools;

use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::Stylize;
use lc3::io;
use lc3::vm::machine::*;
// use vm::machine::*;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use lc3::io::AssemblyInfo;

#[cfg(feature = "asm")]
use crate::asm::codegen::Codegen;
#[cfg(feature = "asm")]
use crate::asm::codegen::lc3tools_codegen::Lc3ToolsCodegen;
#[cfg(feature = "asm")]
use crate::asm::parser::Parser;
#[cfg(feature = "asm")]
use crate::asm::tokenizer::TokenizerErrorInfo;
use crate::cli_tools::get_flag;
#[cfg(feature = "asm")]
use crate::cli_tools::get_param;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // probably a better way to do this
    let args_ref = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
    match args_ref[..] {
        [_, "run"] => println!("Please enter path of object file"),
        [_, "run", path, ..] => run_file(path, &args_ref[3..])?,

        #[cfg(feature = "asm")]
        [_, "asm"] => println!("Please enter path of assembly file"),

        #[cfg(feature = "asm")]
        [_, "asm", path, ..] => asm_file(path, &args_ref[3..])?,

        #[cfg(not(feature = "asm"))]
        [_, "asm", ..] => {
            println!("Assembling is not supported in this build. Please enable the 'asm' feature.")
        }

        _ => {
            println!(
                "
lc3-cli help
Subcommands:
    run <path>\t\t\t Run a assembled object file for the LC-3.
    asm <path> [--verbose|-v] [--output|-o <output_path>]\t Assemble an LC-3 assembly file into an object file compatible with lc3tools.
                "
            );
        }
    }

    Ok(())
}

#[cfg(feature = "asm")]
fn format_tokenizer_error(err: TokenizerErrorInfo, source: &str) -> String {
    let source_cause: Vec<&str> = source
        .lines()
        .skip(err.line.saturating_sub(4))
        .take(8)
        .collect();

    let cause = source_cause.join("\n");

    format!("Error: {err:?}\n\n{cause}")
}

#[cfg(feature = "asm")]
fn asm_file(path: &str, args: &[&str]) -> std::io::Result<()> {
    use crate::asm::tokenizer::{Tokenizer, TokenizerResult};

    let verbose = get_flag(args, "verbose", "v");
    let output_file = get_param(args, "output", "o");

    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let msg = "Tokenizing".green().bold();
    println!("{msg}");

    let tokenizer = Tokenizer::new(&contents);
    let tokens = match tokenizer.tokenize() {
        TokenizerResult::Ok(tokens) => tokens,
        TokenizerResult::Err(err) => {
            println!("{}", format_tokenizer_error(err, &contents).red());

            return Ok(());
        }
        TokenizerResult::Fallthrough => panic!("Internal error during tokenization."),
    };

    if verbose {
        println!("tokens: {tokens:#?}");
    }

    let msg = "Parsing".green().bold();
    println!("{msg}");

    let parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            // TODO! better errors
            println!("Error: {err:?}");
            return Ok(());
        }
    };

    if verbose {
        println!("AST: {ast:#?}");
    }

    let msg = "Assembling".green().bold();
    println!("{msg}");

    let codegen = Lc3ToolsCodegen::new();
    let result = codegen.generate(ast);

    if let Some(output_file) = output_file {
        let mut file = File::create(&output_file)?;
        file.write_all(&result.bytes)?;

        let msg = format!(
            "{} {} {}",
            "Assembly finished".green().bold(),
            ">>".grey(),
            output_file.green()
        );
        println!("{msg}");
    } else {
        println!(
            "No output specified, sending to stdio:\n{:#?}",
            result.bytes
        );
    }

    Ok(())
}

fn run_file(path: &str, args: &[&str]) -> std::io::Result<()> {
    let AssemblyInfo { data } = io::read_file(Path::new(path));

    let ip = cli_tools::get_param(args, "pc", None).unwrap_or("3000".to_string());
    let ip = u16::from_str_radix(&ip, 16)
        .expect("Invalid hex for starting instruction pointer/program counter position.");

    let mut machine = Machine::new(ip, true, true, &[]);

    for datum in data {
        machine.set_span_at(datum.orig, &datum.data);
    }

    crossterm::terminal::enable_raw_mode()?;

    while !machine.halted {
        if crossterm::event::poll(Duration::ZERO)? {
            let event = crossterm::event::read()?;

            if let Some(key_event) = event.as_key_event() {
                if key_event.code == KeyCode::Char('c')
                    && key_event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                } else if let Some(char) = key_event.code.as_char() {
                    machine.set_keyboard_key(char as u16);
                }
            }
        }

        if let Some(data) = machine.poll_display_data() {
            crossterm::terminal::disable_raw_mode()?; // lol, maybe dont do this?
            print!("{}", data as u8 as char);
            crossterm::terminal::enable_raw_mode()?;
            std::io::stdout().flush()?;
        }

        machine.step();
    }

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
