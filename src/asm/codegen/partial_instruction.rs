use std::collections::HashMap;

use lc3::vm::instructions::{DesiredConditionFlags, Instruction};

use crate::parser::Operand;

// TODO not a very DRY solution, consider refactoring where the parser emits the Instructions directly.
// It will require some way to find all the label offsets before hand though and build a map of those.
// Or it could build it as it emits the Instructions and then go back and fix offsets.

#[derive(Debug)]
pub struct PartialInstruction {
    pub opcode: String, // todo replace with enum, or maybe don't create a PartialInstruction at all and just make a Instruction from the parsing step? FIXME
    pub operands: Vec<Operand>,
}

impl PartialInstruction {

    fn get_label_or_offset(&self, operand: usize, abs_position: usize, label_lookup: &HashMap<String, usize>) -> Option<i16> {
        match self.operands[operand] {
            Operand::Label(ref name) => {
                let label_pos = *label_lookup.get(name).expect("Label does not exist");
                let desired_pos = (label_pos as isize) - (abs_position) as isize;

                Some(desired_pos as i16)
            }

            Operand::Number(num) => Some(num),

            _ => None, // TODO better errors
        }

    }

    // TODO better errors (although the parsing process should have handled all the cases here)
    pub fn as_u16(&self, abs_position: usize, label_lookup: &HashMap<String, usize>) -> Option<u16> {
        if self.opcode.starts_with("br") {
            let negative = self.opcode.contains('n');
            let zero = self.opcode.contains('n');
            let positive = self.opcode.contains('n');
        
            let flags = DesiredConditionFlags {
                negative,
                zero,
                positive,
            };

            let Some(desired_pos) = self.get_label_or_offset(0, abs_position, label_lookup) else {
                return None;
            };

            return Some(Instruction::Branch(flags, (desired_pos).into()).encode());

        }
        
        match self.opcode.as_str() { // maybe merge this with the parsing step
            "add" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None
                };

                let Operand::Register(s1) = self.operands[1] else {
                    return None
                };

                if let Operand::Register(s2) = self.operands[2] {
                    Some(Instruction::Add(dst, s1, s2).encode())
                } else if let Operand::Number(num) = self.operands[2] {
                    Some(Instruction::AddImmediate(dst, s1, num.into()).encode())
                } else {
                    None
                }
            }

            "and" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None
                };

                let Operand::Register(s1) = self.operands[1] else {
                    return None
                };

                if let Operand::Register(s2) = self.operands[2] {
                    Some(Instruction::And(dst, s1, s2).encode())
                } else if let Operand::Number(num) = self.operands[2] {
                    Some(Instruction::AndImmediate(dst, s1, num.into()).encode())
                } else {
                    None
                }
            }

            "jmp" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None
                };

                Some(Instruction::Jump(dst).encode())
            }

            "jsr" => {
                let Some(desired_pos) = self.get_label_or_offset(0, abs_position, label_lookup) else {
                    return None;
                };

                Some(Instruction::JumpSubroutine((desired_pos).into()).encode())
            }

            "jsrr" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None;
                };

                Some(Instruction::JumpSubroutineRegister(dst).encode())
            }

            "ld" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None;
                };

                let Some(desired_pos) = self.get_label_or_offset(1, abs_position, label_lookup) else {
                    return None;
                };

                Some(Instruction::Load(dst, (desired_pos).into()).encode())
            }

            "ldi" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None;
                };

                let Some(desired_pos) = self.get_label_or_offset(1, abs_position, label_lookup) else {
                    return None;
                };

                Some(Instruction::LoadIndirect(dst, (desired_pos).into()).encode())
            }

            "ldr" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None;
                };

                let Operand::Register(baser) = self.operands[0] else {
                    return None;
                };

                let Operand::Number(offset6) = self.operands[0] else {
                    return None;
                };

                Some(Instruction::LoadRegister(dst, baser, (offset6).into()).encode())
            }

            "lea" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None;
                };

                let Some(desired_pos) = self.get_label_or_offset(1, abs_position, label_lookup) else {
                    return None;
                };

                Some(Instruction::LoadEffectiveAddress(dst, (desired_pos).into()).encode())
            }

            "not" => {
                let Operand::Register(dst) = self.operands[0] else {
                    return None;
                };

                let Operand::Register(s1) = self.operands[0] else {
                    return None;
                };

                Some(Instruction::Not(dst, s1).encode())
            }

            // parser does not emit ret

            "rti" => {
                Some(Instruction::ReturnFromInterrupt.encode())
            }
            
            "st" => {
                let Operand::Register(sr) = self.operands[0] else {
                    return None;
                };

                let Some(desired_pos) = self.get_label_or_offset(1, abs_position, label_lookup) else {
                    return None;
                };

                Some(Instruction::Store(sr, (desired_pos).into()).encode())
            }

            "sti" => {
                let Operand::Register(sr) = self.operands[0] else {
                    return None;
                };

                let Some(desired_pos) = self.get_label_or_offset(1, abs_position, label_lookup) else {
                    return None;
                };

                Some(Instruction::StoreIndirect(sr, (desired_pos).into()).encode())
            }

            "str" => {
                let Operand::Register(sr) = self.operands[0] else {
                    return None;
                };

                let Operand::Register(baser) = self.operands[0] else {
                    return None;
                };

                let Operand::Number(offset6) = self.operands[0] else {
                    return None;
                };

                Some(Instruction::StoreRegister(sr, baser, (offset6).into()).encode())
            }

            "trap" => {
                let Operand::Number(vector) = self.operands[0] else {
                    return None;
                };

                Some(Instruction::Trap(vector as u8).encode())
            }

            _ => None,
        }
    }
}

impl PartialInstruction {
    pub fn new(opcode: String, operands: Vec<Operand>) -> PartialInstruction {
        Self { opcode, operands }
    }
}