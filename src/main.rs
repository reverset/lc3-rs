mod bit_util;

#[cfg(test)]
mod tests;

mod vm;

mod io;

use vm::machine::*;

use core::panic;
use std::path::Path;

use crate::io::AssemblyInfo;

fn get_position(args: &[&str], long: &str, short: Option<&str>) -> Option<usize> {
    args.iter()
        .enumerate()
        .find(|(_, v)| {
            let v = v.trim();
            if v == format!("--{long}").as_str() {
                true
            } else if let Some(short) = short {
                v == format!("-{short}").as_str()
            } else {
                false
            }
        })
        .map(|(i, _)| i)
}

// TODO, trying to avoid using any more libraries than necessary
#[allow(unused)] // lol
fn get_flag(args: &[&str], long: &str, short: Option<&str>) -> bool {
    if args.contains(&format!("--{long}").as_str()) {
        true
    } else if let Some(short) = short {
        args.contains(&format!("-{short}").as_str())
    } else {
        false
    }
}

fn get_param(args: &[&str], long: &str, short: Option<&str>) -> Option<String> {
    if let Some(pos) = get_position(args, long, short) {
        if pos+1 >= args.len() {
            panic!("--{long}/-{short:?} requires an input after.");
        }
        Some(args[pos+1].to_string())
    } else {
        None
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // probably a better way to do this
    let args_ref = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
    match args_ref[..] {
        [_, "run"] => {
            println!("Please enter path of object file");
        }
        [_, "run", path, ..] => {
            let AssemblyInfo { data } = io::read_file(Path::new(path));
            
            let ip = get_param(&args_ref, "pc", None).unwrap_or("3000".to_string());
            let ip = u16::from_str_radix(&ip, 16).expect("Invalid hex for starting instruction pointer/program counter position.");
            
            let mut machine = Machine::new(std::io::stdin(), std::io::stdout(), ip, &[]);

            for datum in data {
                let instrs: Vec<i16> = datum.data.iter().map(|x| *x as i16).collect();
                machine.set_span_at(datum.orig, &instrs[..]);
            }


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
