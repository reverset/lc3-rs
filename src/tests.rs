use std::io::{BufReader, BufWriter};
use super::*;

#[test]
fn add_instr() {
    let add = Instruction::add(Register::R0, Register::R1, Register::R2);
    let add_imm = Instruction::add_imm(Register::R0, Register::R1, 5);

    assert_eq!(format!("{add}"), "0b0001000001000010");
    assert_eq!(format!("{add_imm}"), "0b0001000001100101");

    assert!(add.is_add());
    assert!(add_imm.is_add());
}

#[test]
fn and_instr() {
    let and = Instruction::and(Register::R0, Register::R1, Register::R2);
    let and_imm = Instruction::and_imm(Register::R0, Register::R1, 5);

    assert_eq!(format!("{and}"), "0b0101000001000010");
    assert_eq!(format!("{and_imm}"), "0b0101000001100101");

    assert!(and.is_and());
    assert!(and_imm.is_and());
}

#[test]
fn not_instr() {
    let not = Instruction::not(Register::R0, Register::R1);

    assert_eq!(format!("{not}"), "0b1001000001111111");
}

#[test]
fn add_add() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
        Instruction::add_imm(Register::R1, Register::R0, 5), // r1 = 10
    ]);

    machine.step();
    machine.step();

    assert_eq!(machine.registers.get(Register::R0), 5);
    assert_eq!(machine.registers.get(Register::R1), 10);
}

#[test]
fn add_add_and() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
        Instruction::add_imm(Register::R1, Register::R1, 5), // r1 = 5
        Instruction::and(Register::R2, Register::R0, Register::R1), // r2 = 5 (r0 & r1)
    ]);

    machine.step();
    machine.step();
    machine.step();

    assert_eq!(machine.registers.get(Register::R0), 5);
    assert_eq!(machine.registers.get(Register::R1), 5);
    assert_eq!(machine.registers.get(Register::R2), 5);
}

#[test]
fn add_not() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
        Instruction::not(Register::R1, Register::R0),        // r2 = 1111111111111010 = -6 (!r0)
    ]);

    machine.step();
    machine.step();

    assert_eq!(machine.registers.get(Register::R0), 5);
    assert_eq!(machine.registers.get(Register::R1), -6);
}

#[test]
fn print_a() {
    let mut output = BufWriter::new(Vec::new());

    let mut machine = Machine::new(
        std::io::stdin(),
        &mut output,
        0x3000,
        &[
            // largest immediate we can do is 7
            // yes this can be condensed
            Instruction::add_imm(Register::R0, Register::R1, 7), // r0 = 7
            Instruction::add_imm(Register::R1, Register::R1, 7), // r1 = 7
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (14)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (21)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (28)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (35)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (42)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (49)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (56)
            Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (63)
            Instruction::add_imm(Register::R0, Register::R0, 2), // r0 = r0 + 2 (65, 'A' in ASCII)
            Instruction::trap_out(),                             // print r0
            Instruction::trap_halt(),
        ],
    );

    machine.run_until_halt();
    drop(machine);

    let buf = output.into_inner().unwrap();
    let output = String::from_utf8(buf).unwrap();

    assert_eq!(output, "A");
}

#[test]
fn check_branching() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 7), // r0 = 7 // flag = p
        Instruction::branch(0b001, 1), // check if positive, then skip over the next instruction
        Instruction::trap_halt(),
        Instruction::add_imm(Register::R0, Register::R0, 7), // r0 = 14
        Instruction::trap_halt(),
    ]);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 14);

    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 7), // r0 = 7 // flag = p
        Instruction::branch(0b110, 1), // check if negative or zero (false), so we don't jump
        Instruction::trap_halt(),
        Instruction::add_imm(Register::R0, Register::R0, 7), // r0 = 14
        Instruction::trap_halt(),
    ]);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 7);

    let mut machine = Machine::new_std(&[
        Instruction::branch(0b111, -1), // check if negative or zero (false), so we don't jump
    ]);

    machine.step();

    assert_eq!(machine.ip, 0x3000);
}

#[test]
fn check_jmp() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 4), // r0 = 4
        Instruction::jmp(Register::R0),                      // jmp to 4 (r0 = 4)
        Instruction::trap_halt(), // this should not happen since we jumped over it
    ]);

    machine.step();
    machine.step();

    assert_eq!(machine.ip, 4);
}

#[test]
fn check_ld() {
    let mut machine = Machine::new_std(&[
        Instruction::ld(Register::R0, -2), // r0 = 50 (see code after this)
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(0x3000 - 1, 50);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 50);
}

#[test]
fn hello_world() {
    let mut output = BufWriter::new(Vec::new());

    let mut machine = Machine::new(
        std::io::stdin(),
        &mut output,
        0x3000,
        &[
            Instruction::lea(Register::R0, 2), // r0 = text_addr
            Instruction::trap_puts(),          // print string stored at address in r0
            Instruction::trap_halt(),
        ],
    );

    let text = "Hello, world!\n";
    let text_addr = 0x3003;
    machine.string_set(text_addr, text);

    machine.run_until_halt();

    drop(machine);

    assert_eq!(
        String::from_utf8(output.into_inner().unwrap()).unwrap(),
        text
    );
}

#[test]
fn check_ldi() {
    let mut machine = Machine::new_std(&[
        Instruction::ldi(Register::R0, -2), // r0 = 20 (load value stored at the address stored in ip offset -2)
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(1, 20);
    machine.set_memory_at(0x3000 - 1, 1);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 20);
}

#[test]
fn check_ldr() {
    let mut machine = Machine::new_std(&[
        Instruction::ld(Register::R0, -2),
        Instruction::ldr(Register::R1, Register::R0, 0),
        Instruction::ldr(Register::R2, Register::R0, 1),
        Instruction::ldr(Register::R3, Register::R0, 2),
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(0x3000 - 1, 10);
    machine.set_span_at(10, &[1, 2, 3]);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R1), 1);
    assert_eq!(machine.registers.get(Register::R2), 2);
    assert_eq!(machine.registers.get(Register::R3), 3);
}

#[test]
fn check_jsr() {
    let mut machine = Machine::new_std(&[
        Instruction::jsr(3),
        Instruction::add_imm(Register::R5, Register::R0, 1),
        Instruction::add_imm(Register::R5, Register::R0, 1),
        Instruction::add_imm(Register::R5, Register::R0, 1),
        Instruction::add_imm(Register::R0, Register::R1, 5),
        Instruction::trap_halt(),
    ]);

    machine.run_until_halt();
    println!("{:?}", machine.registers);

    assert_eq!(machine.registers.get(Register::R0), 5);
    assert_ne!(machine.registers.get(Register::R5), 3);
    assert_eq!(machine.registers.get(Register::R7), 0x3001);
}

#[test]
fn check_jsrr() {
    let mut machine = Machine::new_std(&[
        Instruction::ld(Register::R1, -2),
        Instruction::jsrr(Register::R1),
        Instruction::add_imm(Register::R5, Register::R0, 1),
        Instruction::add_imm(Register::R5, Register::R0, 1),
        Instruction::add_imm(Register::R5, Register::R0, 1),
        Instruction::add_imm(Register::R0, Register::R2, 5),
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(0x3000 - 1, 0x3005);
    machine.step();
    machine.step();
    machine.step();
    // machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 5);
    assert_ne!(machine.registers.get(Register::R5), 3);
    assert_eq!(machine.registers.get(Register::R7), 0x3002);
}

#[test]
fn check_st() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 5),
        Instruction::st(Register::R0, -3),
        Instruction::trap_halt(),
    ]);

    machine.run_until_halt();

    assert_eq!(machine.memory[0x3000 - 1], 5);
}

#[test]
fn check_sti() {
    let mut machine = Machine::new_std(&[
        Instruction::add_imm(Register::R0, Register::R1, 5),
        Instruction::sti(Register::R0, -3),
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(0x3000 - 1, 0x2000);

    machine.run_until_halt();

    assert_eq!(machine.memory[0x2000], 5);
}

#[test]
fn check_str() {
    let mut machine = Machine::new_std(&[
        Instruction::ld(Register::R0, -2),
        Instruction::add_imm(Register::R1, Register::R5, 5),
        Instruction::add_imm(Register::R2, Register::R5, 6),
        Instruction::str(Register::R1, Register::R0, 0),
        Instruction::str(Register::R2, Register::R0, 1),
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(0x3000 - 1, 0x2000);

    machine.run_until_halt();

    assert_eq!(machine.memory[0x2000], 5);
    assert_eq!(machine.memory[0x2001], 6);
}

#[test]
fn hello_world_5() {
    // adapted from https://github.com/paul-nameless/lc3-asm/blob/master/tests/hello2.asm
    let mut output = BufWriter::new(Vec::new());

    let mut machine = Machine::new(
        std::io::stdin(),
        &mut output,
        0x3000,
        &[
            Instruction::lea(Register::R0, 5),
            Instruction::ld(Register::R1, 19),
            Instruction::trap_puts(),
            Instruction::add_imm(Register::R1, Register::R1, -1),
            Instruction::branch(0b001, -3),
            Instruction::trap_halt(),
        ],
    );

    let text = "Hello, World!\n";
    machine.string_set(0x3006, text);
    machine.set_memory_at(1 + 0x3006 + (text.len() as u16), 5); // 1 + ... because of null byte

    machine.run_until_halt();
    drop(machine);

    assert_eq!(
        String::from_utf8(output.into_inner().unwrap()).unwrap(),
        text.repeat(5)
    );
}

#[test]
fn test_getc() {
    let data = [0b0000111u8; 1];
    let input = BufReader::new(&data[..]);

    let mut machine = Machine::new(
        input,
        std::io::stdout(),
        0x3000,
        &[Instruction::trap_get_c(), Instruction::trap_halt()],
    );

    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 0b0000111);
}
