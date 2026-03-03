use std::{fs::File, io::Read, path::Path};

pub mod read_complex;
pub mod read_raw;

const LC3_OBJ_HEADER: &[u8] = b"LC-3 OBJ FILE";

pub struct AssemblyInfo { // TODO, debug info, linker info, etc
    pub orig: u16,
    pub data: Vec<u16>,
}

pub fn read_file(path: &Path) -> AssemblyInfo {
    let mut buf = Vec::new();

    let mut file = File::open(path).expect("File could not be opened.");

    _ = file.read_to_end(&mut buf).expect("Failed to read file.");

    if buf.starts_with(LC3_OBJ_HEADER) {
        read_complex::read(&buf)
    } else {
        println!("File missing header, interpreting as a raw file.\n");
        read_raw::read(&buf)
    }
}