pub mod instructions;

pub use instructions::Instruction;

use std::{fmt::{self, Write}, mem};

#[derive(Debug, Clone)]
pub struct Executable {
    pub num_consts: Vec<f64>,
    pub code: Vec<u8>,
}

impl Executable {
    pub fn new(num_consts: Vec<f64>, code: Vec<u8>) -> Self {
        Executable { num_consts, code }
    }
}

impl fmt::Display for Executable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "\n<Executable>
NUM_CONSTS: {:?}
    
INSTRUCTIONS:\n",
            self.num_consts
        ))?;

        let code = &self.code;
        let mut i = 0;

        while i < code.len() {
            let instruction: Instruction = unsafe { mem::transmute(code[i]) };
            i += 1;

            f.write_fmt(format_args!("    {:?}", instruction))?;

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
                },
                Instruction::LoadConstNumW => {
                    let index = u16::from_le_bytes([code[i], code[i+1]]);
                    let val = self.num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                    i += 2;
                },
                Instruction::LoadConstNumWW => {
                    let index = u32::from_le_bytes([code[i], code[i+1], code[i+2], code[i+3]]);
                    let val = self.num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                    i += 4;
                },
                Instruction::GetLocal | Instruction::SetLocal => {
                    f.write_fmt(format_args!("    {}", code[i]))?;
                    i += 1;
                }
                Instruction::GetLocalW | Instruction::SetLocalW => {
                    let index = u16::from_le_bytes([code[i], code[i+1]]);
                    f.write_fmt(format_args!("    {}", index))?;
                    i += 12
                }

                _ => {}
            } 


            f.write_char('\n')?;
        }
        f.write_str("</Executable>")?;
        Ok(())
    }
}
