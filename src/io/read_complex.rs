use crate::io::{AssemblyInfo, DataInfo};

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
    // let mut instructions = vec![];

    let mut section = ObjectFileSection::None;

    let mut data_sections: Vec<DataInfo> = vec![];

    let lines: Vec<&str> = data.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line = line.trim().to_lowercase();

        // TODO FIXME!! this is very broken
        // first line is a orig block
        // next line is how many instructions there are (in decimal form)
        // after reading that many instructions, we are at a new orig block
        // TODO IMPLEMENT ABOVE ^^^^ i need to go to sleep

        if line.starts_with(".") {
            section = get_section(&line);
        } else if !line.is_empty() {
            match section {
                ObjectFileSection::Text => {
                    if i + 1 < lines.len() && lines[i+1].len() < 4 && !lines[i+1].is_empty() {
                        println!("line: {}", lines[i+1]);
                        let orig = line.parse::<u16>().unwrap();
                        println!("?? {}", orig);
                        data_sections.push(DataInfo { orig, data: vec![] });
                    } else if lines[i].len() < 4 {
                        continue;
                    } else {
                        let val = u16::from_str_radix(&line, 16).unwrap();
                        data_sections.last_mut().unwrap().data.push(val);
                    }
                }
                
                // TODO
                _ => (),
            }
        }
    }
    println!("sect: {data_sections:?}");
    AssemblyInfo { data: data_sections }
}
