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

    let mut skip_next = false;
    let mut orig_length: u16 = 0;

    for (i, line) in lines.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        let line = line.trim().to_lowercase();

        if line.starts_with(".") {
            section = get_section(&line);
        } else if !line.is_empty() {
            match section {
                ObjectFileSection::Text => {
                    if orig_length > 0 {
                        orig_length -= 1;

                        let val = u16::from_str_radix(&line, 16).unwrap();
                        data_sections.last_mut().unwrap().data.push(val);
                    } else {
                        let orig = u16::from_str_radix(&line, 16).unwrap();
                        data_sections.push(DataInfo { orig, data: vec![] });
                        orig_length = lines[i+1].parse::<u16>().unwrap();
                        skip_next = true;
                    }
                }
                
                // TODO
                _ => (),
            }
        }
    }
    AssemblyInfo { data: data_sections }
}
