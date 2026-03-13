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

#[cfg(test)] // TODO fix input with new keyboard system
mod tests {
    use crate::io::read_complex::read;
    use crate::vm::machine::Machine;
    use std::io::BufWriter;

    #[test]
    fn read_hello() {
         let program = r#"
LC-3 OBJ FILE

.TEXT
3000
68
E019
F022
F020
243F
1202
0404
E004
F022
127F
03FD
F025
0048
0065
006C
006C
006F
002C
0020
0057
006F
0072
006C
0064
0021
000A
0000
0048
006F
0077
0020
006D
0061
006E
0079
0020
0074
0069
006D
0065
0073
0020
0028
0031
0020
0063
0068
0061
0072
0020
0070
006C
0065
0061
0073
0065
0029
0020
0028
0030
002E
002E
003D
0039
0029
003A
0020
0000
FFD0

.SYMBOL
ADDR | EXT | LABEL
3007 |   0 | LOOP
300A |   0 | END
300B |   0 | HELLO
301A |   0 | PROMPT
3043 |   0 | NUM_OFFSET

.LINKER_INFO

.DEBUG
# DEBUG SYMBOLS FOR LC3TOOLS

LABEL      | INDEX
LOOP       |   125
END        |   161
HELLO      |   171
PROMPT     |   204
NUM_OFFSET |   264
====================
LINE | ADDR | SOURCE
   0 | ???? | .orig x3000\n
   1 | ???? | \n
   2 | ???? | ; comment\n
   3 | ???? | \n
   4 | 3000 | LEA R0, PROMPT\n
   5 | 3001 | PUTS\n
   6 | 3002 | GETC\n
   7 | ???? | \n
   8 | 3003 | LD R2, NUM_OFFSET\n
   9 | 3004 | ADD R1, R0, R2\n
  10 | ???? | \n
  11 | 3005 | BRz END\n
  12 | ???? | \n
  13 | 3006 | LEA R0, HELLO ; another comment\n
  14 | 3007 | LOOP PUTS\n
  15 | 3008 | ADD R1, R1, #-1\n
  16 | 3009 | BRp LOOP\n
  17 | ???? | \n
  18 | ???? | END\n
  19 | 300A | HALT\n
  20 | ???? | \n
  21 | 300B | HELLO .stringz \"Hello, World!\\n\"\n
  22 | 301A | PROMPT .stringz \"How many times (1 char please) (0..=9): \"\n
  23 | ???? | \n
  24 | 3043 | NUM_OFFSET .fill #-48\n
  25 | ???? | \n
  26 | ???? | .end
====================

         "#;

        let bytes = program.as_bytes();

        let asm_info = read(bytes);


        // let mut input = BufReader::new(&input_buf[..]);
        let mut output = BufWriter::new(Vec::new());

        let mut machine = Machine::new(std::io::stdin(), &mut output, 0x3000, &[]);

        for datum in asm_info.data {
            let instrs: Vec<i16> = datum.data.iter().map(|x| *x as i16).collect();
            machine.set_span_at(datum.orig, &instrs[..]);
        }

        let input_buf: [u16; 2] = ['5' as u16, '\n' as u16];
        let mut next_input: usize = 0;
        // machine.run_until_halt();
        while !machine.halted {
            if next_input < input_buf.len() && machine.set_keyboard_key(input_buf[next_input]) {
                next_input = next_input + 1;
            }
            machine.step();
        }

        drop(machine);

        let output = output.into_inner().unwrap();
        let res = String::from_utf8(output).unwrap();
        assert_eq!(res, format!("How many times (1 char please) (0..=9): {}", "Hello, World!\n".repeat(5)));
    }
}
