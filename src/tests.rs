use super::*;
use std::io::{BufReader, BufWriter};

use crate::vm::instructions::*;

#[test]
fn add_instr() {
    let add = Instruction::Add(Register::R0, Register::R1, Register::R2).encode();
    let add_imm = Instruction::AddImmediate(Register::R0, Register::R1, 5.into()).encode();

    assert_eq!(format!("{add:016b}"), "0001000001000010");
    assert_eq!(format!("{add_imm:016b}"), "0001000001100101");
}

#[test]
fn and_instr() {
    let and = Instruction::And(Register::R0, Register::R1, Register::R2).encode();
    let and_imm = Instruction::AndImmediate(Register::R0, Register::R1, 5.into()).encode();

    assert_eq!(format!("{and:016b}"), "0101000001000010");
    assert_eq!(format!("{and_imm:016b}"), "0101000001100101");
}

#[test]
fn not_instr() {
    let not = Instruction::Not(Register::R0, Register::R1).encode();

    assert_eq!(format!("{not:016b}"), "1001000001111111");
}

#[test]
fn add_add() {
    let mut machine = Machine::new_std(&[
        Instruction::AddImmediate(Register::R0, Register::R1, 5.into()), // r0 = 5
        Instruction::AddImmediate(Register::R1, Register::R0, 5.into()), // r1 = 10
    ]);

    machine.step();
    machine.step();

    assert_eq!(machine.registers.get(Register::R0), 5);
    assert_eq!(machine.registers.get(Register::R1), 10);
}

#[test]
fn add_add_and() {
    let mut machine = Machine::new_std(&[
        Instruction::AddImmediate(Register::R0, Register::R1, 5.into()), // r0 = 5
        Instruction::AddImmediate(Register::R1, Register::R1, 5.into()), // r1 = 5
        Instruction::And(Register::R2, Register::R0, Register::R1),      // r2 = 5 (r0 & r1)
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
        Instruction::AddImmediate(Register::R0, Register::R1, 5.into()), // r0 = 5
        Instruction::Not(Register::R1, Register::R0), // r2 = 1111111111111010 = -6 (!r0)
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
            Instruction::AddImmediate(Register::R0, Register::R1, 7.into()), // r0 = 7
            Instruction::AddImmediate(Register::R1, Register::R1, 7.into()), // r1 = 7
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (14)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (21)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (28)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (35)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (42)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (49)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (56)
            Instruction::Add(Register::R0, Register::R0, Register::R1),      // r0 = r0 + r1 (63)
            Instruction::AddImmediate(Register::R0, Register::R0, 2.into()), // r0 = r0 + 2 (65, 'A' in ASCII)
            Instruction::trap_out(),                                         // print r0
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
        Instruction::AddImmediate(Register::R0, Register::R1, 7.into()), // r0 = 7 // flag = p
        Instruction::Branch(0b001.into(), 1.into()), // check if positive, then skip over the next instruction
        Instruction::trap_halt(),
        Instruction::AddImmediate(Register::R0, Register::R0, 7.into()), // r0 = 14
        Instruction::trap_halt(),
    ]);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 14);

    let mut machine = Machine::new_std(&[
        Instruction::AddImmediate(Register::R0, Register::R1, 7.into()), // r0 = 7 // flag = p
        Instruction::Branch(0b110.into(), 1.into()), // check if negative or zero (false), so we don't jump
        Instruction::trap_halt(),
        Instruction::AddImmediate(Register::R0, Register::R0, 7.into()), // r0 = 14
        Instruction::trap_halt(),
    ]);
    machine.run_until_halt();

    assert_eq!(machine.registers.get(Register::R0), 7);

    let mut machine = Machine::new_std(&[
        Instruction::Branch(0b111.into(), (-1).into()), // check if negative or zero (false), so we don't jump
    ]);

    machine.step();

    assert_eq!(machine.ip, 0x3000);
}

#[test]
fn check_branch_bits() {
    let branch = Instruction::Branch(0b111.into(), (-1).into()).encode();

    assert_eq!(format!("{branch:016b}"), "0000111111111111");

    let branch = Instruction::decode(branch);

    assert_eq!(branch, Instruction::Branch(0b111.into(), (-1).into()));
}

#[test]
fn check_jmp() {
    let mut machine = Machine::new_std(&[
        Instruction::AddImmediate(Register::R0, Register::R1, 4.into()), // r0 = 4
        Instruction::Jump(Register::R0),                                 // jmp to 4 (r0 = 4)
        Instruction::trap_halt(), // this should not happen since we jumped over it
    ]);

    machine.step();
    machine.step();

    assert_eq!(machine.ip, 4);
}

#[test]
fn check_ld() {
    let mut machine = Machine::new_std(&[
        Instruction::Load(Register::R0, (-2).into()), // r0 = 50 (see code after this)
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
            Instruction::LoadEffectiveAddress(Register::R0, 2.into()), // r0 = text_addr
            Instruction::trap_puts(), // print string stored at address in r0
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
        Instruction::LoadIndirect(Register::R0, (-2).into()), // r0 = 20 (load value stored at the address stored in ip offset -2)
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
        Instruction::Load(Register::R0, (-2).into()),
        Instruction::LoadRegister(Register::R1, Register::R0, 0.into()),
        Instruction::LoadRegister(Register::R2, Register::R0, 1.into()),
        Instruction::LoadRegister(Register::R3, Register::R0, 2.into()),
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
        Instruction::JumpSubroutine(3.into()),
        Instruction::AddImmediate(Register::R5, Register::R0, 1.into()),
        Instruction::AddImmediate(Register::R5, Register::R0, 1.into()),
        Instruction::AddImmediate(Register::R5, Register::R0, 1.into()),
        Instruction::AddImmediate(Register::R0, Register::R1, 5.into()),
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
        Instruction::Load(Register::R1, (-2).into()),
        Instruction::JumpSubroutineRegister(Register::R1),
        Instruction::AddImmediate(Register::R5, Register::R0, 1.into()),
        Instruction::AddImmediate(Register::R5, Register::R0, 1.into()),
        Instruction::AddImmediate(Register::R5, Register::R0, 1.into()),
        Instruction::AddImmediate(Register::R0, Register::R2, 5.into()),
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
        Instruction::AddImmediate(Register::R0, Register::R1, 5.into()),
        Instruction::Store(Register::R0, (-3).into()),
        Instruction::trap_halt(),
    ]);

    machine.run_until_halt();

    assert_eq!(machine.memory[0x3000 - 1], 5);
}

#[test]
fn check_sti() {
    let mut machine = Machine::new_std(&[
        Instruction::AddImmediate(Register::R0, Register::R1, 5.into()),
        Instruction::StoreIndirect(Register::R0, (-3).into()),
        Instruction::trap_halt(),
    ]);

    machine.set_memory_at(0x3000 - 1, 0x2000);

    machine.run_until_halt();

    assert_eq!(machine.memory[0x2000], 5);
}

#[test]
fn check_str() {
    let mut machine = Machine::new_std(&[
        Instruction::Load(Register::R0, (-2).into()),
        Instruction::AddImmediate(Register::R1, Register::R5, 5.into()),
        Instruction::AddImmediate(Register::R2, Register::R5, 6.into()),
        Instruction::StoreRegister(Register::R1, Register::R0, 0.into()),
        Instruction::StoreRegister(Register::R2, Register::R0, 1.into()),
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
            Instruction::LoadEffectiveAddress(Register::R0, 5.into()),
            Instruction::Load(Register::R1, 19.into()),
            Instruction::trap_puts(),
            Instruction::AddImmediate(Register::R1, Register::R1, (-1).into()),
            Instruction::Branch(0b001.into(), (-3).into()),
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
