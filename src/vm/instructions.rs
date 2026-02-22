use crate::bit_util::{i5_to_i8, i6_to_i8, i9_to_i16, i11_to_i16};
use crate::vm::instructions::Instruction::{
    Add, AddImmediate, And, AndImmediate, Branch, Jump, JumpSubroutine, JumpSubroutineRegister,
    Load, LoadEffectiveAddress, LoadIndirect, LoadRegister, Not, Reserved, ReturnToInterrupt,
    Store, StoreIndirect, StoreRegister, Trap,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl From<Register> for u8 {
    fn from(val: Register) -> u8 {
        val as u8
    }
}

impl From<Register> for usize {
    fn from(val: Register) -> usize {
        val as usize
    }
}

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        match value {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,
            _ => panic!("Invalid register"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Immediate5(i8);

impl Immediate5 {
    pub fn into_inner(self) -> i8 {
        self.0
    }
}

impl From<i16> for Immediate5 {
    fn from(value: i16) -> Self {
        let value = value & 0b11111;
        Immediate5(i5_to_i8(value as i8))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PcOffset9(i16);

impl PcOffset9 {
    pub fn into_inner(self) -> i16 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PcOffset11(i16);

impl PcOffset11 {
    pub fn into_inner(self) -> i16 {
        self.0
    }
}

impl From<i16> for PcOffset11 {
    fn from(value: i16) -> Self {
        let value = value & 0b11111111111;
        PcOffset11(i11_to_i16(value))
    }
}

impl From<i16> for PcOffset9 {
    fn from(value: i16) -> Self {
        let value = value & 0b111111111;
        PcOffset9(i9_to_i16(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Offset6(i8);

impl Offset6 {
    pub fn into_inner(self) -> i8 {
        self.0
    }
}

impl From<i16> for Offset6 {
    fn from(value: i16) -> Self {
        let value = value & 0b111111;
        Offset6(i6_to_i8(value as i8))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DesiredConditionFlags {
    pub negative: bool,
    pub zero: bool,
    pub positive: bool,
}

impl DesiredConditionFlags {
    pub fn into_flags(self) -> u8 {
        let mut result = 0u8;

        if self.negative {
            result |= 0b100;
        }

        if self.zero {
            result |= 0b010;
        }

        if self.positive {
            result |= 0b001;
        }

        result
    }
}

impl From<u16> for DesiredConditionFlags {
    // assumes that only the first 3 bits are set
    fn from(value: u16) -> Self {
        let negative = value & 0b100 != 0;
        let zero = value & 0b010 != 0;
        let positive = value & 0b001 != 0;

        Self {
            negative,
            zero,
            positive,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Add(Register, Register, Register),
    AddImmediate(Register, Register, Immediate5),
    And(Register, Register, Register),
    AndImmediate(Register, Register, Immediate5),
    Branch(DesiredConditionFlags, PcOffset9),
    Jump(Register),
    JumpSubroutine(PcOffset11),
    JumpSubroutineRegister(Register),
    Load(Register, PcOffset9),
    LoadIndirect(Register, PcOffset9),
    LoadRegister(Register, Register, Offset6),
    LoadEffectiveAddress(Register, PcOffset9),
    Not(Register, Register),
    // RET is just a special case of JMP,
    ReturnToInterrupt, // TODO
    Store(Register, PcOffset9),
    StoreIndirect(Register, PcOffset9),
    StoreRegister(Register, Register, Offset6),
    Trap(u8),
    Reserved,
}

impl Instruction {
    pub fn decode(instr: u16) -> Self {
        let header = Self::get_header(instr);

        match header {
            // ADD & AND
            kind @ (0b0001 | 0b0101) => {
                let dr = (instr >> 9) & 0b111;
                let sr1 = (instr >> 6) & 0b111;
                let is_immediate = ((instr >> 5) & 0b1) != 0;

                if is_immediate {
                    if kind == 0b001 {
                        AddImmediate(dr.into(), sr1.into(), (instr as i16).into())
                    } else {
                        AndImmediate(dr.into(), sr1.into(), (instr as i16).into())
                    }
                } else {
                    let sr2 = instr & 0b111;
                    if kind == 0b001 {
                        Add(dr.into(), sr1.into(), sr2.into())
                    } else {
                        And(dr.into(), sr1.into(), sr2.into())
                    }
                }
            }

            // branch
            0b0000 => {
                let flags = (instr >> 9) & 0b111;
                let pcoffset: PcOffset9 = (instr as i16).into();

                Branch(flags.into(), pcoffset)
            }

            // jump
            0b1100 => {
                let baser = (instr >> 6) & 0b111;

                Jump(baser.into())
            }

            // jump sub && jump sub register
            0b0100 => {
                let is_jsr = ((instr >> 11) & 0b1) != 0;
                if is_jsr {
                    JumpSubroutine((instr as i16).into())
                } else {
                    let baser = (instr >> 6) & 0b111;

                    JumpSubroutineRegister(baser.into())
                }
            }

            // load & load indirect
            kind @ (0b0010 | 0b1010) => {
                let dr = (instr >> 9) & 0b111;
                let pcoffset9: PcOffset9 = (instr as i16).into();

                if kind == 0b0010 {
                    Load(dr.into(), pcoffset9)
                } else {
                    LoadIndirect(dr.into(), pcoffset9)
                }
            }

            // load register
            0b0110 => {
                let dr = (instr >> 9) & 0b111;
                let baser = (instr >> 6) & 0b111;
                let offset6: Offset6 = (instr as i16).into();

                LoadRegister(dr.into(), baser.into(), offset6)
            }

            // load effective address
            0b1110 => {
                let dr = (instr >> 9) & 0b111;
                let pcoffset9: PcOffset9 = (instr as i16).into();

                LoadEffectiveAddress(dr.into(), pcoffset9)
            }

            // not
            0b1001 => {
                let dr = (instr >> 9) & 0b111;
                let sr = (instr >> 6) & 0b111;

                Not(dr.into(), sr.into())
            }

            // RET is just jmp in disguise

            // return to interrupt
            0b1000 => ReturnToInterrupt,

            // store & store indirect
            kind @ (0b0011 | 0b1011) => {
                let sr = (instr >> 9) & 0b111;
                let pcoffet9: PcOffset9 = (instr as i16).into();

                if kind == 0b0011 {
                    Store(sr.into(), pcoffet9)
                } else {
                    StoreIndirect(sr.into(), pcoffet9)
                }
            }

            // store register
            0b0111 => {
                let sr = (instr >> 9) & 0b111;
                let baser = (instr >> 6) & 0b111;
                let offset6: Offset6 = (instr as i16).into();

                StoreRegister(sr.into(), baser.into(), offset6)
            }

            // TRAP
            0b1111 => {
                let trapvector8 = instr as u8;

                Trap(trapvector8)
            }

            // reserved
            0b1101 => Reserved,

            _ => {
                panic!("Invalid opcode!");
            }
        }
    }

    pub fn get_header(instr: u16) -> u8 {
        ((instr >> 12) & 0b1111) as u8
    }

    // source for the following trap vectors: https://acg.cis.upenn.edu/milom/cse240-Fall05/handouts/Ch09-a.pdf
    pub fn trap_get_c() -> Self {
        Trap(0x20)
    }

    pub fn trap_out() -> Self {
        Trap(0x21)
    }

    pub fn trap_puts() -> Self {
        Trap(0x22)
    }

    pub fn trap_in() -> Self {
        Trap(0x23)
    }

    // TODO TRAP 0x24 (putsp)

    pub fn trap_halt() -> Self {
        Trap(0x25)
    }
}

impl Instruction {
    pub fn encode(self) -> u16 {
        match self {
            Add(dest, s1, s2) => {
                let dr = dest as u16;
                let sr1 = s1 as u16;
                let sr2 = s2 as u16;

                // 15-12  11-9  8-6         2-0
                // 0001    DR   SR1  0  00  SR2

                let mut instr: u16 = 1 << 12;

                instr |= dr << 9;
                instr |= sr1 << 6;
                instr |= sr2;

                instr
            }

            AddImmediate(dest, s1, imm5) => {
                let dr = dest as u16;
                let sr1 = s1 as u16;

                let mut instr: u16 = 1 << 12;

                instr |= dr << 9;
                instr |= sr1 << 6;

                instr |= 1 << 5;

                instr |= (imm5.into_inner() & 0b11111) as u16;

                instr
            }

            And(dest, s1, s2) => {
                let dr = dest as u16;
                let sr1 = s1 as u16;
                let sr2 = s2 as u16;

                let mut instr: u16 = (1 << 14) | (1 << 12);

                instr |= dr << 9;
                instr |= sr1 << 6;
                instr |= sr2;

                instr
            }

            AndImmediate(dest, s1, imm5) => {
                let dr = dest as u16;
                let sr1 = s1 as u16;

                let mut instr: u16 = 0b0101 << 12;

                instr |= dr << 9;
                instr |= sr1 << 6;

                instr |= 1 << 5;

                instr |= (imm5.into_inner() & 0b11111) as u16;

                instr
            }

            Branch(flags, offset) => {
                let mut instr: u16 = ((flags.into_flags() & 0b111) as u16) << 9;

                instr |= (offset.into_inner() & 0b111111111) as u16;

                instr
            }

            Jump(register) => {
                let mut instr: u16 = 0b1100 << 12;

                instr |= (register as u16) << 6;

                instr
            }

            JumpSubroutine(offset) => {
                let mut instr: u16 = 0b0100 << 12;

                instr |= 1 << 11;
                instr |= (offset.into_inner() & 0b11111111111) as u16;

                instr
            }

            JumpSubroutineRegister(register) => {
                let mut instr: u16 = 0b0100 << 12;

                instr |= (register as u16) << 6;

                instr
            }

            Load(dest, offset) => {
                let mut instr: u16 = 0b0010 << 12;

                instr |= (dest as u16) << 9;
                instr |= (offset.into_inner() & 0b111111111) as u16;

                instr
            }

            LoadIndirect(dest, offset) => {
                let mut instr: u16 = 0b1010 << 12;

                instr |= (dest as u16) << 9;
                instr |= (offset.into_inner() & 0b111111111) as u16;

                instr
            }

            LoadRegister(dest, baser, offset) => {
                let mut instr: u16 = 0b0110 << 12;

                instr |= (dest as u16) << 9;
                instr |= (baser as u16) << 6;
                instr |= (offset.into_inner() & 0b111111) as u16;

                instr
            }

            LoadEffectiveAddress(dest, offset) => {
                let mut instr: u16 = 0b1110 << 12;

                instr |= (dest as u16) << 9;
                instr |= (offset.into_inner() & 0b111111111) as u16;

                instr
            }

            Not(dest, source) => {
                let mut instr: u16 = 0b1001 << 12;

                instr |= (dest as u16) << 9;
                instr |= (source as u16) << 6;
                instr |= 0b111111;

                instr
            }

            ReturnToInterrupt => 0b1000 << 12,

            Store(source, offset) => {
                let mut instr: u16 = 0b0011 << 12;
                instr |= (source as u16) << 9;
                instr |= (offset.into_inner() & 0b111111111) as u16;

                instr
            }

            StoreIndirect(source, offset) => {
                let mut instr: u16 = 0b1011 << 12;
                instr |= (source as u16) << 9;
                instr |= (offset.into_inner() & 0b111111111) as u16;

                instr
            }

            StoreRegister(source, baser, offset) => {
                let mut instr: u16 = 0b0111 << 12;
                instr |= (source as u16) << 9;
                instr |= (baser as u16) << 6;
                instr |= (offset.into_inner() & 0b111111) as u16;

                instr
            }

            Trap(vector) => {
                let mut instr: u16 = 0b1111 << 12;

                instr |= vector as u16;

                instr
            }

            Reserved => 0b1101 << 12,
        }
    }
}

#[derive(Default, Debug)]
pub struct Registers {
    reg: [i16; 8],
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,

            _ => panic!("Invalid register: {}", value), // todo print machine state
        }
    }
}

impl Registers {
    pub fn get(&self, i: Register) -> i16 {
        let i: usize = i.into();
        self.reg[i]
    }

    pub fn get_mut(&mut self, i: Register) -> &mut i16 {
        let i: usize = i.into();
        &mut self.reg[i]
    }
}
