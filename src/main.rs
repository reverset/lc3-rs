mod bit_util;

#[cfg(test)]
mod tests;

mod vm;

use vm::instructions::*;
use vm::machine::*;

use std::fs::File;
use std::io::Read;
use std::path::Path;

fn read_binary_file(file: &Path) -> Vec<Instruction> {
    let mut res = Vec::new();

    let mut file = File::open(file).unwrap();
    loop {
        let mut buf = [0u8; 2];
        if file.read_exact(&mut buf).is_err() {
            break;
        }
        let instr = Instruction(((buf[0] as i16) << 8) | ((buf[1] as i16) & 0b11111111));
        res.push(instr);
    }

    res
}

fn main() {
    // maybe use clap or something ...

    let args: Vec<String> = std::env::args().collect();

    // probably a better way to do this
    let args_ref = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    match args_ref[..] {
        [_, "run"] => {
            println!("Please enter path of binary file");
        }
        [_, "run", path] => {
            let binary = read_binary_file(Path::new(path));

            let orig = binary[0].0;

            let mut machine = Machine::new(
                std::io::stdin(),
                std::io::stdout(),
                orig as u16,
                &binary[1..],
            );

            machine.run_until_halt();
        }

        _ => {
            println!(
                "
lc3-rs help
Subcommands:
    run <path>\t\t\t Run a assembled binary file for the LC-3.
                "
            );
        }
    }
}
