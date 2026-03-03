use crate::io::AssemblyInfo;

enum ObjectFileSection {
    Text,
    Symbol,
    LinkerInfo,
    Debug,

    None,
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


pub fn read(data: &[u8]) -> AssemblyInfo {
    let data = String::from_utf8(data.to_vec()).expect("File contained invalid UTF-8, even though header stated LC-3 OBJ FILE");
    let mut instructions = vec![];

    let mut section = ObjectFileSection::None;
    let mut orig = None;

    for line in data.lines() {
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
                
                // TODO
                _ => (),
            }
        }
    }

    AssemblyInfo { orig: orig.unwrap(), data: instructions }
}
