mod bit_util;

#[cfg(test)]
mod tests;

mod vm;

use vm::machine::*;

use std::fs::File;
use std::io::ErrorKind::InvalidData;
use std::io::Read;
use std::path::Path;

fn read_file(path: &Path) -> (u16, Vec<u16>) {
    let mut file = File::open(path).unwrap();

    // yes the file gets opened 2 times, FIXME
    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => {
            drop(file);
            if buf.starts_with("LC-3 OBJ FILE") {
                read_lc3_object_file(path)
            } else {
                read_binary_file(path)
            }
        },
        Err(e) => {
            match e.kind() {
                InvalidData => {
                    drop(file);
                    read_binary_file(path)
                }
                _=> panic!("There was an error reading the file: {e:?}"),
            }
        },
    }

}

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

enum ObjectFileSection {
    Text,
    Symbol,
    LinkerInfo,
    Debug,

    None,
}

fn read_lc3_object_file(file: &Path) -> (u16, Vec<u16>) {
    let mut file = File::open(file).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    let mut instructions = vec![];

    let mut section = ObjectFileSection::None;
    let mut orig = None;

    for line in buf.lines() {
        let line = line.trim().to_lowercase();

        if line.starts_with(".") {
            section = get_section(&line);
        } else if !line.is_empty() {
            match section {
                ObjectFileSection::Text => {
                    let val = u16::from_str_radix(&line, 16).unwrap();
                    if orig.is_none() {
                        orig = Some(val);
                    } else {
                        instructions.push(val);
                    }

                }

                _ => (),
            }
        }
    }

    (orig.unwrap(), instructions)
}

fn get_section(line: &str) -> ObjectFileSection {
    match line {
        ".text" => ObjectFileSection::Text,
        ".symbol" => ObjectFileSection::Symbol,
        ".linker_info" => ObjectFileSection::LinkerInfo,
        ".debug" => ObjectFileSection::Debug,
        _ => panic!("Unknown section: {}", line),
    }
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
            let (orig, binary) = read_file(Path::new(path));

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
