mod bit_util;

#[cfg(test)]
mod tests;

mod vm;

mod io;

use vm::machine::*;

use std::path::Path;

use crate::io::AssemblyInfo;

// TODO, trying to avoid using any more libraries than necessary
fn get_flag(args: Vec<&str>, long: &str, short: Option<&str>) -> bool {
    if args.contains(&format!("--{long}").as_str()) {
        true
    } else if let Some(short) = short {
        args.contains(&format!("-{short}").as_str())
    } else {
        false
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
        [_, "run", path] => {
            let AssemblyInfo { orig, data: binary } = io::read_file(Path::new(path));

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
