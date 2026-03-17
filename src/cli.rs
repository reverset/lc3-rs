#![cfg(feature = "cli")]

use crossterm::event::{KeyCode, KeyModifiers};
use lc3::io;
use lc3::vm::machine::*;
// use vm::machine::*;

use core::panic;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use lc3::io::AssemblyInfo;

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
        if pos + 1 >= args.len() {
            panic!("--{long}/-{short:?} requires an input after.");
        }
        Some(args[pos + 1].to_string())
    } else {
        None
    }
}

fn main() -> std::io::Result<()> {
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
            let ip = u16::from_str_radix(&ip, 16)
                .expect("Invalid hex for starting instruction pointer/program counter position.");

            let mut machine = Machine::new(ip, true, &[]);

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
        }

        _ => {
            println!(
                "
lc3-cli help
Subcommands:
    run <path>\t\t\t Run a assembled object file for the LC-3.
                "
            );
        }
    }

    Ok(())
}
