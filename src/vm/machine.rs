use crate::bit_util::convert_str_to_i16_vec;
use crate::vm::instructions::Instruction::{
    Add, AddImmediate, And, AndImmediate, Branch, Jump, JumpSubroutine, JumpSubroutineRegister,
    Load, LoadEffectiveAddress, LoadIndirect, LoadRegister, Not, Reserved, ReturnFromInterrupt,
    Store, StoreIndirect, StoreRegister, Trap,
};
use crate::vm::instructions::{DesiredConditionFlags, Instruction, Register, Registers};
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::ops::{Index, IndexMut};

const KBSR: u16 = 0xFE00;
const KBDR: u16 = 0xFE02;

const DSR: u16 = 0xFE04;
const DDR: u16 = 0xFE06;
const PSR: u16 = 0xFFFC;

const MCR: u16 = 0xFFFE;

const PRIVILEGE_EXC: u8 = 0x0;
const ILLEGAL_OPCODE_EXC: u8 = 0x1;
const ACV_EXC: u8 = 0x2; // illegal access to protected memory

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lc3Error {
    IllegalMemoryAccess(u16),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConditionCode {
    Negative,
    Zero,
    Positive,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum PrivilegeMode {
    Supervisor,

    #[default]
    User,
}

impl PrivilegeMode {
    pub fn is_supervisor(&self) -> bool {
        *self == PrivilegeMode::Supervisor
    }
}

impl ConditionCode {
    pub fn into_flags(self) -> u8 {
        match self {
            ConditionCode::Negative => 0b100,
            ConditionCode::Zero => 0b010,
            ConditionCode::Positive => 0b001,
        }
    }
}

pub struct Memory(HashMap<u16, i16>);

impl Memory {
    #[allow(unused)]
    fn entries(&self) -> Keys<'_, u16, i16> {
        self.0.keys()
    }
}

impl Index<u16> for Memory {
    type Output = i16;

    fn index(&self, index: u16) -> &Self::Output {
        self.0.get(&index).unwrap_or(&0)
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.0.entry(index).or_insert(0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MemoryModificationEvent {
    Read(i16),
    Write(i16),
}

pub struct Machine<'a> {
    pub registers: Registers,
    pub memory: Memory,
    pub ip: u16, // LC-3 is word addressable.

    // perhaps separate this into a PSR struct
    // include interrupt enable bit?
    pub condition_code: ConditionCode,
    pub privilege: PrivilegeMode,
    pub priority: u8,

    pub halted: bool,

    pub protect_system_memory: bool,
    pub protect_device_memory: bool,

    memory_event_callbacks: HashMap<u16, fn(&mut Self, MemoryModificationEvent)>, // maybe a different data structure or hashing algorithm
}

// Not sure if the condition code should start as the Zero flag.
// according to https://www.cs.utexas.edu/~fussell/courses/cs310h/lectures/Lecture_10-310h.pdf it states
// that exactly one condition code is set at all times. I suppose Zero is a sensible default.
#[allow(unused)]
impl<'a> Machine<'a> {
    pub fn new_x3000(instructions: &[Instruction]) -> Self {
        Self::new(0x3000, true, true, instructions)
    }

    pub fn new(
        pc: u16,
        protect_system_memory: bool,
        protect_device_memory: bool,
        instructions: &[Instruction],
    ) -> Self {
        // let mut memory = Vec::from_iter((0..orig).map(|_| 0));
        // for inst in instructions {
        //     memory.push(inst.encode() as i16);
        // }

        let mut memory = HashMap::new();
        for (i, instruction) in instructions.iter().enumerate() {
            memory.insert(pc + i as u16, instruction.encode() as i16);
        }

        let mut machine = Self {
            registers: Registers::default(),
            memory: Memory(memory),
            ip: pc,
            condition_code: ConditionCode::Zero,
            privilege: PrivilegeMode::User,
            priority: 0,
            halted: false,
            protect_system_memory,
            protect_device_memory,

            memory_event_callbacks: HashMap::new(),
        };

        machine.load_basic_os();

        machine
    }

    pub fn load_basic_os(&mut self) {
        self.set_memory_at_unchecked(0x0100 + ILLEGAL_OPCODE_EXC as u16, 0x0200);
        self.set_span_at(
            0x0200,
            &[
                LoadEffectiveAddress(Register::R0, (2).into()).encode() as i16,
                Instruction::trap_puts().encode() as i16,
                Instruction::trap_halt().encode() as i16,
            ],
        );
        self.string_set(0x0203, "[exc] Illegal opcode\n\0");

        self.set_memory_at_unchecked(0x0100 + PRIVILEGE_EXC as u16, 0x219);
        self.set_span_at(
            0x219,
            &[
                LoadEffectiveAddress(Register::R0, (2).into()).encode() as i16,
                Instruction::trap_puts().encode() as i16,
                Instruction::trap_halt().encode() as i16,
            ],
        );
        self.string_set(0x21c, "[exc] invalid privilege\n\0");

        self.set_memory_at_unchecked(0x0100 + ACV_EXC as u16, 0x235);
        self.set_span_at(
            0x235,
            &[
                LoadEffectiveAddress(Register::R0, (2).into()).encode() as i16,
                Instruction::trap_puts().encode() as i16,
                Instruction::trap_halt().encode() as i16,
            ],
        );
        self.string_set(0x238, "[exc] ACV\n\0");

        // automatically reset status bit after a read
        self.add_io_callback(KBDR, |machine, event| {
            if let MemoryModificationEvent::Read(_) = event {
                machine.memory[KBSR] &= !(1 << 15); // clear 15th bit
            }
        });

        self.set_keyboard_interrupts(true);

        self.add_io_callback(DDR, |machine, event| {
            if let MemoryModificationEvent::Write(_) = event {
                machine.memory[DSR] &= !(1 << 15); // clear 15th bit
            }
        });

        // GETC trap vector
        self.set_memory_at_unchecked(0x20, 0x0244);
        self.set_span_at(
            0x0244,
            &[
                LoadIndirect(Register::R0, 93.into()).encode() as i16,
                Branch(
                    DesiredConditionFlags {
                        positive: true,
                        zero: true,
                        negative: false,
                    },
                    (-2).into(),
                )
                .encode() as i16,
                LoadIndirect(Register::R0, 92.into()).encode() as i16,
                ReturnFromInterrupt.encode() as i16,
            ],
        );

        self.set_memory_at_unchecked(0x02A2, KBSR as i16);
        self.set_memory_at_unchecked(0x02A3, KBDR as i16);

        // OUT trap vector
        self.set_memory_at_unchecked(0x21, 0x0248);
        self.set_span_at(
            0x0248,
            &[
                // move stack pointer
                AddImmediate(Register::R6, Register::R6, (-1).into()).encode() as i16,
                // push R0 onto stack
                StoreRegister(Register::R0, Register::R6, (0).into()).encode() as i16,
                // load DSR into R0
                LoadIndirect(Register::R0, (0x02A4 - (0x0248 + 2) - 1).into()).encode() as i16,
                Branch(
                    DesiredConditionFlags {
                        negative: false,
                        zero: true,
                        positive: true,
                    },
                    (-2).into(),
                )
                .encode() as i16,
                // pop data from stack
                LoadRegister(Register::R0, Register::R6, (0).into()).encode() as i16,
                AddImmediate(Register::R6, Register::R6, (1).into()).encode() as i16,
                StoreIndirect(Register::R0, (0x02A5 - (0x0248 + 6) - 1).into()).encode() as i16,
                ReturnFromInterrupt.encode() as i16,
            ],
        );

        self.set_memory_at_unchecked(0x02A4, DSR as i16);
        self.set_memory_at_unchecked(0x02A5, DDR as i16);

        self.set_display_status(true);

        // PUTS trap vector

        self.set_memory_at_unchecked(0x22, 0x0250);
        self.set_span_at(
            0x0250,
            &[
                // push R0 onto stack
                AddImmediate(Register::R6, Register::R6, (-1).into()).encode() as i16,
                StoreRegister(Register::R0, Register::R6, (0).into()).encode() as i16,
                // push R1 onto stack
                AddImmediate(Register::R6, Register::R6, (-1).into()).encode() as i16,
                StoreRegister(Register::R1, Register::R6, (0).into()).encode() as i16,
                // copy R0 to R1
                AddImmediate(Register::R1, Register::R0, (0).into()).encode() as i16,
                // load char
                LoadRegister(Register::R0, Register::R1, 0.into()).encode() as i16,
                // if zero we jump to end
                Branch(
                    DesiredConditionFlags {
                        negative: false,
                        zero: true,
                        positive: false,
                    },
                    (3).into(),
                )
                .encode() as i16,
                // otherwise we print first char
                Instruction::trap_out().encode() as i16,
                // update pointer to get address of next char
                AddImmediate(Register::R1, Register::R1, (1).into()).encode() as i16,
                // jump back up to load register
                Branch(
                    DesiredConditionFlags {
                        negative: true,
                        zero: true,
                        positive: true,
                    },
                    (-5).into(),
                )
                .encode() as i16,
                // after the loop
                // pop off R1
                LoadRegister(Register::R1, Register::R6, (0).into()).encode() as i16,
                AddImmediate(Register::R6, Register::R6, (1).into()).encode() as i16,
                // pop off R0
                LoadRegister(Register::R0, Register::R6, (0).into()).encode() as i16,
                AddImmediate(Register::R6, Register::R6, (1).into()).encode() as i16,
                ReturnFromInterrupt.encode() as i16,
            ],
        );

        // IN trap vector
        self.set_memory_at_unchecked(0x23, 0x025f);
        self.set_span_at(
            0x025f,
            &[
                Instruction::trap_puts().encode() as i16,
                Instruction::trap_get_c().encode() as i16,
                Instruction::trap_out().encode() as i16,
                ReturnFromInterrupt.encode() as i16,
            ],
        );

        // Machine Control Register
        // set 15th bit to 1.
        self.set_memory_at_unchecked(MCR, 1 << 15);

        self.add_io_callback(MCR, |machine, event| {
            if let MemoryModificationEvent::Write(value) = event
                && value >= 0
            {
                // 15th bit is cleared
                // time to halt
                machine.halted = true;
            }
        });
    }

    pub fn interrupt(&mut self, vector: u8, urgency: u8) {
        if urgency < self.priority {
            return;
        }

        let vector = (vector as u16) + 0x0100;

        let addr = self.get_memory_at_unchecked(vector);
        if addr == 0 {
            return;
        }

        // kinda duplicated code, maybe fix
        let psr = self.encode_psr();

        self.set_privilege(PrivilegeMode::Supervisor);
        self.priority = urgency;

        let pc = self.ip;

        self.stack_push(psr as i16);
        self.stack_push(pc as i16);

        self.ip = addr as u16;
    }

    // true => data set
    // false => flag cleared
    pub fn get_keyboard_status(&self) -> bool {
        self.memory[KBSR] < 0
    }

    pub fn get_display_status(&self) -> bool {
        self.memory[DSR] < 0 // 15th bit is set
    }

    pub fn set_keyboard_key(&mut self, data: u16) -> bool {
        if !self.get_keyboard_status() {
            self.memory[KBDR] = data as i16;
            self.memory[KBSR] |= (1 << 15); // 15th bit is set.

            if self.get_keyboard_interrupt_enable_bit() {
                self.interrupt(0x80, 4);
            }

            true
        } else {
            false
        }
    }

    pub fn get_keyboard_interrupt_enable_bit(&self) -> bool {
        ((self.memory[KBSR] >> 14) & 0b1) == 1
    }

    pub fn set_keyboard_interrupts(&mut self, enable: bool) {
        let mask = ((enable as i16) & 0b1) << 14;
        self.memory[KBSR] &= !mask;
        self.memory[KBSR] |= mask;
    }

    pub fn get_display_data(&self) -> u16 {
        self.memory[DDR] as u16
    }

    pub fn set_display_status(&mut self, ready: bool) {
        let ready = ready as u16;
        self.memory[DSR] = (self.memory[KBSR] & !(1 << 15)) | (ready << 15) as i16;
    }

    pub fn get_display_interrupt_enable_bit(&self) -> bool {
        ((self.memory[DSR] >> 14) & 0b1) == 1
    }

    pub fn poll_display_data(&mut self) -> Option<u16> {
        if !self.get_display_status() {
            self.set_display_status(true);
            let data = self.get_display_data();
            Some(data)
        } else {
            None
        }
    }

    pub fn set_privilege(&mut self, privilege: PrivilegeMode) {
        self.privilege = privilege;
        self.registers.mode = self.privilege;
    }

    pub fn is_address_in_io_section(&self, address: u16) -> bool {
        address >= 0xFE00
    }

    pub fn is_address_in_system_section(&self, address: u16) -> bool {
        address <= 0x2FFF
    }

    pub fn is_address_protected(&self, address: u16) -> bool {
        (self.protect_device_memory && self.is_address_in_io_section(address))
            || (self.protect_system_memory && self.is_address_in_system_section(address))
    }

    // Set data in the IO section of memory (0xFE00 to 0xFFFF)
    // index is the offset from 0xFE00. 0 to 511
    pub fn set_device_data(&mut self, index: u16, data: i16) {
        let desired = index + 0xFE00;
        self.memory[desired] = data;
    }

    pub fn invoke_io_event(&mut self, address: u16, event: MemoryModificationEvent) {
        let Some(callback) = self.memory_event_callbacks.get(&address) else {
            return;
        };

        callback(self, event);
    }

    pub fn add_io_callback(&mut self, address: u16, func: fn(&mut Self, MemoryModificationEvent)) {
        self.memory_event_callbacks.insert(address, func);
    }

    pub fn set_memory_at_unchecked(&mut self, address: u16, value: i16) {
        self.memory[address] = value;
    }

    pub fn get_memory_at_unchecked(&mut self, address: u16) -> i16 {
        self.memory[address]
    }

    pub fn set_memory_at(&mut self, index: u16, value: i16) -> Result<(), Lc3Error> {
        if self.privilege == PrivilegeMode::User && self.is_address_protected(index) {
            return Err(Lc3Error::IllegalMemoryAccess(index));
        }

        if index == PSR {
            self.decode_psr(value as u16);
            return Ok(());
        }

        self.memory[index] = value;

        if self.is_address_in_io_section(index) {
            self.invoke_io_event(index, MemoryModificationEvent::Write(value));
        }

        Ok(())
    }

    pub fn get_memory_at(&mut self, index: u16) -> Result<i16, Lc3Error> {
        if self.privilege == PrivilegeMode::User && self.is_address_protected(index) {
            return Err(Lc3Error::IllegalMemoryAccess(index));
        }

        if index == PSR {
            return Ok(self.encode_psr() as i16);
        }

        let val = self.memory[index];

        if self.is_address_in_io_section(index) {
            self.invoke_io_event(index, MemoryModificationEvent::Read(val));
        }

        Ok(val)
    }

    pub fn set_span_at(&mut self, index: u16, value: &[i16]) {
        for (value_index, i) in (index..(index + value.len() as u16)).enumerate() {
            self.memory[i] = value[value_index];
        }
    }

    pub fn string_set(&mut self, index: u16, value: &str) {
        self.set_span_at(index, &convert_str_to_i16_vec(value));
    }

    pub fn run_until_halt(&mut self) {
        while !self.halted {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let instr = self.memory[self.ip];
        self.ip += 1; // ip points to the next instruction

        if let Err(err) = self.evaluate(Instruction::decode(instr as u16)) {
            match err {
                Lc3Error::IllegalMemoryAccess(_) => self.interrupt(ACV_EXC, 7),
            }
        }
    }

    pub fn add_to_ip(&mut self, offset: i16) {
        self.ip.wrapping_add_signed(offset);
    }

    pub fn encode_psr(&self) -> u16 {
        let mut res: u16 = 0;
        if self.privilege == PrivilegeMode::User {
            res |= 1 << 15;
        }
        let cond_codes = self.condition_code.into_flags();

        res |= (cond_codes & 0b111) as u16;

        res |= (self.priority as u16 & 0b111) << 8;

        res
    }

    pub fn decode_psr(&mut self, psr: u16) {
        let privilege = psr >> 15;
        if privilege == 0 {
            self.privilege = PrivilegeMode::Supervisor;
        } else {
            self.privilege = PrivilegeMode::User;
        }

        let cond_codes = psr & 0b111;
        self.priority = ((psr >> 8) & 0b111) as u8;

        self.condition_code = match cond_codes {
            0b100 => ConditionCode::Negative,
            0b010 => ConditionCode::Zero,
            0b001 => ConditionCode::Positive,

            _ => panic!(
                "invalid condition code found in SSP while decoding new PSR. {:03b}",
                cond_codes
            ),
        }
    }

    // cleanup needed
    pub fn evaluate(&mut self, instr: Instruction) -> Result<(), Lc3Error> {
        match instr {
            Add(dest, s1, s2) => {
                let s1 = self.registers.get(s1);
                let s2 = self.registers.get(s2);

                *self.registers.get_mut(dest) = s1.wrapping_add(s2);
                self.set_condition_code_based_on(dest);
            }

            AddImmediate(dest, s1, imm5) => {
                let s1 = self.registers.get(s1);

                *self.registers.get_mut(dest) = s1.wrapping_add(imm5.into_inner() as i16);
                self.set_condition_code_based_on(dest);
            }

            And(dest, s1, s2) => {
                let s1 = self.registers.get(s1);
                let s2 = self.registers.get(s2);

                *self.registers.get_mut(dest) = s1 & s2;
                self.set_condition_code_based_on(dest);
            }

            AndImmediate(dest, s1, imm5) => {
                let s1 = self.registers.get(s1);

                *self.registers.get_mut(dest) = s1 & (imm5.into_inner() as i16);
                self.set_condition_code_based_on(dest);
            }

            Branch(flags, offset) => {
                if self.condition_code.into_flags() & flags.into_flags() != 0 {
                    self.ip = self.ip.wrapping_add_signed(offset.into_inner());
                }
            }

            Jump(register) => {
                self.ip = self.registers.get(register) as u16;
            }

            JumpSubroutine(offset) => {
                *self.registers.get_mut(Register::R7) = self.ip as i16;
                self.ip = self.ip.wrapping_add_signed(offset.into_inner());
            }

            JumpSubroutineRegister(baser) => {
                *self.registers.get_mut(Register::R7) = self.ip as i16;
                self.ip = self.registers.get(baser) as u16;
            }

            Load(dest, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                let val = self.get_memory_at(addr)?;

                *self.registers.get_mut(dest) = val;
                self.set_condition_code_based_on(dest);
            }

            LoadIndirect(dest, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                let addr = self.get_memory_at(addr)? as u16;

                let val = self.get_memory_at(addr)?;
                *self.registers.get_mut(dest) = val;
                self.set_condition_code_based_on(dest);
            }

            LoadRegister(dest, baser, offset) => {
                let addr = (self.registers.get(baser) as u16)
                    .wrapping_add_signed(offset.into_inner() as i16);
                let val = self.get_memory_at(addr)?;

                *self.registers.get_mut(dest) = val;
                self.set_condition_code_based_on(dest);
            }

            LoadEffectiveAddress(dest, offset) => {
                let ea = self.ip.wrapping_add_signed(offset.into_inner());

                *self.registers.get_mut(dest) = ea as i16;
                self.set_condition_code_based_on(dest);
            }

            Not(dest, source) => {
                let source = self.registers.get(source);

                *self.registers.get_mut(dest) = !source;
                self.set_condition_code_based_on(dest);
            }

            // RET is just JMP
            ReturnFromInterrupt => {
                if self.privilege.is_supervisor() {
                    self.ip = self.stack_pop() as u16;
                    let psr = self.stack_pop() as u16;

                    self.decode_psr(psr);
                } else {
                    self.interrupt(PRIVILEGE_EXC, 7);
                }
            }

            Store(source, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                self.set_memory_at(addr, self.registers.get(source))?;
            }

            StoreIndirect(source, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                let addr = self.get_memory_at(addr)? as u16;

                self.set_memory_at(addr, self.registers.get(source))?;
            }

            StoreRegister(source, baser, offset) => {
                let addr = self.registers.get(baser) as u16;
                let addr = addr.wrapping_add_signed(offset.into_inner() as i16);

                self.set_memory_at(addr, self.registers.get(source))?;
            }

            Trap(vector) => self.handle_trap(vector),

            Reserved => self.interrupt(ILLEGAL_OPCODE_EXC, 7),
        };

        Ok(())
    }

    fn handle_trap(&mut self, vec: u8) {
        // TODO, implement trap vectors in the Machine's instructions itself,
        // instead of implementing it within Rust
        match vec {
            // 0x23 => todo!("in"),
            0x24 => todo!("putsp"),

            // halt
            0x25 => {
                // technically this should modify the MCR, but whatever
                self.halted = true;
            }
            vector => {
                // this part is not implemented according to the ISA pdf,
                // but rather the book 'Introduction To Computing Systems: From Bits & Gates To C/C++ & Beyond (3rd Edition)'

                let psr = self.encode_psr();

                self.set_privilege(PrivilegeMode::Supervisor);

                let pc = self.ip;

                self.stack_push(psr as i16);
                self.stack_push(pc as i16);

                let desired = self.memory[vector as u16];
                self.ip = desired as u16;
            }
        }
    }

    pub fn stack_push(&mut self, val: i16) {
        let original_val = self.registers.get(Register::R6) as u16;
        let desired_val = original_val.wrapping_sub(1); // stack grows down, so to push to sub 1

        *self.registers.get_mut(Register::R6) = desired_val as i16;

        self.set_memory_at_unchecked(desired_val, val);
    }

    pub fn stack_pop(&mut self) -> i16 {
        let original_val = self.registers.get(Register::R6) as u16;
        let new_val = original_val.wrapping_add(1); // stack grows down, so to pop we add one.

        *self.registers.get_mut(Register::R6) = new_val as i16;
        self.get_memory_at_unchecked(original_val)
    }

    fn set_condition_code_based_on(&mut self, reg: Register) {
        self.condition_code = match self.registers.get(reg) {
            0 => ConditionCode::Zero,
            1.. => ConditionCode::Positive,
            ..0 => ConditionCode::Negative,
        }
    }
}
