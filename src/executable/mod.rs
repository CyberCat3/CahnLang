mod instructions;

pub use instructions::Instruction;

use std::{
    fmt::{self, Write},
    mem,
};

use crate::compiler::lexical_analysis::TokenPos;

#[derive(Debug, Clone)]
pub struct Executable {
    pub num_consts: Vec<f64>,
    pub code: Vec<u8>,
    pub code_map: Vec<TokenPos>,
    pub source_file: String,
    pub string_data: String,
}

impl Executable {
    pub fn new(
        num_consts: Vec<f64>,
        string_data: String,

        source_file: String,
        code: Vec<u8>,
        code_map: Vec<TokenPos>,
    ) -> Self {
        Executable {
            string_data,
            source_file,
            num_consts,
            code,
            code_map,
        }
    }
}

impl fmt::Display for Executable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "\n<Executable>
NUM_CONSTS: {:?}

STRING_DATA: '{}'
    
INSTRUCTIONS:\n",
            self.num_consts, self.string_data,
        ))?;

        let code = &self.code;
        let code_map = &self.code_map;
        let mut i = 0;

        while i < code.len() {
            let instruction: Instruction = unsafe { mem::transmute(code[i]) };
            let code_pos = code_map[i];
            i += 1;

            f.write_fmt(format_args!(
                "{}:{} \t{}\t{:?}",
                self.source_file,
                code_pos,
                i - 1,
                instruction
            ))?;

            match instruction {
                Instruction::LoadLitNum => {
                    f.write_fmt(format_args!("    '{}'", code[i]))?;
                    i += 1;
                }
                Instruction::LoadConstNum => {
                    let index = code[i];
                    let val = self.num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                    i += 1;
                }
                Instruction::LoadConstNumW => {
                    let index = u16::from_le_bytes([code[i], code[i + 1]]);
                    let val = self.num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                    i += 2;
                }
                Instruction::LoadConstNumWW => {
                    let index =
                        u32::from_le_bytes([code[i], code[i + 1], code[i + 2], code[i + 3]]);
                    let val = self.num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                    i += 4;
                }
                Instruction::JumpIfFalse | Instruction::Jump => {
                    let jump_location =
                        u32::from_le_bytes([code[i], code[i + 1], code[i + 2], code[i + 3]]);

                    f.write_fmt(format_args!("    {}", jump_location))?;
                    i += 4;
                }

                Instruction::GetLocal | Instruction::SetLocal => {
                    f.write_fmt(format_args!("    {}", code[i]))?;
                    i += 1;
                }
                Instruction::GetLocalW | Instruction::SetLocalW => {
                    let index = u16::from_le_bytes([code[i], code[i + 1]]);
                    f.write_fmt(format_args!("    {}", index))?;
                    i += 2
                }

                Instruction::LoadStringLiteral => {
                    let start_index =
                        u32::from_le_bytes([code[i], code[i + 1], code[i + 2], code[i + 3]])
                            as usize;

                    let end_index =
                        u32::from_le_bytes([code[i + 4], code[i + 5], code[i + 6], code[i + 7]])
                            as usize;

                    i += 8;

                    f.write_fmt(format_args!(
                        "    {}..{} '{}'",
                        start_index,
                        end_index,
                        &self.string_data[start_index..end_index]
                    ))?;
                }

                Instruction::Add => {}
                Instruction::Mul => {}
                Instruction::Sub => {}
                Instruction::Div => {}
                Instruction::Negate => {}
                Instruction::Not => {}
                Instruction::LoadTrue => {}
                Instruction::LoadFalse => {}
                Instruction::LoadNil => {}
                Instruction::LessThan => {}
                Instruction::GreaterThan => {}
                Instruction::LessThanOrEqual => {}
                Instruction::GreaterThanOrEqual => {}
                Instruction::Equal => {}
                Instruction::Dup => {}
                Instruction::Pop => {}
                Instruction::Print => {}
                Instruction::Concat => {}
            }

            f.write_char('\n')?;
        }
        f.write_str("</Executable>")?;
        Ok(())
    }
}
