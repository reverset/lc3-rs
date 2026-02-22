mod bit_util;

#[cfg(test)]
mod tests;

mod vm;

use vm::machine::*;

use std::fs::File;
use std::io::Read;
use std::path::Path;

fn read_binary_file(file: &Path) -> (u16, Vec<u16>) {
    let mut res = Vec::new();

    let mut file = File::open(file).unwrap();
    let mut orig = None;
    loop {
        let mut buf = [0u8; 2];
        if file.read_exact(&mut buf).is_err() {
            break;
        }
        let value = ((buf[0] as u16) << 8) | ((buf[1] as u16) & 0b11111111);
        if orig.is_none() {
            orig = Some(value);
        } else {
            res.push(value);
        }
    }

    (orig.unwrap(), res)
}

fn main() {
    // maybe use clap or something ...

    let args: Vec<String> = std::env::args().collect();

    // probably a better way to do this
    let args_ref = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    match args_ref[..] {
        [_, "run"] => {
            println!("Please enter path of object file");
        }
        [_, "run", path] => {
            let (orig, binary) = read_binary_file(Path::new(path));

            let mut machine = Machine::new(std::io::stdin(), std::io::stdout(), orig, &[]);

            let bin = binary.iter().map(|x| *x as i16).collect::<Vec<i16>>();

            machine.set_span_at(orig, &bin[..]);

            machine.run_until_halt();
        }

        _ => {
            println!(
                "
lc3-rs help
Subcommands:
    run <path>\t\t\t Run a assembled object file for the LC-3.
                "
            );
        }
    }
}
