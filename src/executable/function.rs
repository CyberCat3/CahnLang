use std::fmt::Write;

use {
    crate::{
        compiler::lexical_analysis::TokenPos,
        executable::{Executable, Instruction},
        utils::PanickingByteBufferReader,
    },
    std::{fmt, mem},
};

#[derive(Debug, Clone, Copy)]
pub enum FunctionName {
    Anonymous,
    Named {
        start_index: usize,
        end_index: usize,
    },
}

impl FunctionName {
    pub fn fmt<'a>(self, string_data: &'a str) -> FormatableFunctionName<'a> {
        FormatableFunctionName {
            name: self,
            string_data,
        }
    }
}

pub struct FormatableFunctionName<'a> {
    name: FunctionName,
    string_data: &'a str,
}

impl<'a> fmt::Display for FormatableFunctionName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self.name {
                FunctionName::Anonymous => "Anonymous",
                FunctionName::Named {
                    start_index,
                    end_index,
                } => &self.string_data[start_index..end_index],
            }
        ))
    }
}

#[derive(Clone)]
pub struct CahnFunction {
    pub param_count: u8,
    pub code: Vec<u8>,
    pub code_map: Vec<TokenPos>,
    pub name: FunctionName,
}

impl CahnFunction {
    fn new_helper(
        param_count: u8,
        code: Vec<u8>,
        code_map: Vec<TokenPos>,
        name: FunctionName,
    ) -> Self {
        Self {
            param_count,
            code,
            code_map,
            name,
        }
    }

    pub fn new(
        param_count: u8,
        code: Vec<u8>,
        code_map: Vec<TokenPos>,
        name_start: usize,
        name_end: usize,
    ) -> Self {
        let name = FunctionName::Named {
            start_index: name_start,
            end_index: name_end,
        };
        Self::new_helper(param_count, code, code_map, name)
    }

    pub fn new_anonymous(param_count: u8, code: Vec<u8>, code_map: Vec<TokenPos>) -> Self {
        Self::new_helper(param_count, code, code_map, FunctionName::Anonymous)
    }

    pub fn fmt<'a>(&'a self, exec: &'a Executable) -> FormatableCahnFunction<'a> {
        FormatableCahnFunction { func: self, exec }
    }
}

pub struct FormatableCahnFunction<'a> {
    func: &'a CahnFunction,
    exec: &'a Executable,
}

impl<'a> fmt::Display for FormatableCahnFunction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.func.name {
            FunctionName::Anonymous => f.write_fmt(format_args!("<fn:{}>", self.func.param_count)),
            FunctionName::Named {
                start_index: _,
                end_index: _,
            } => f.write_fmt(format_args!(
                "<fn {}:{}>",
                self.func.name.fmt(&self.exec.string_data),
                self.func.param_count,
            )),
        }
    }
}

impl<'a> fmt::Debug for FormatableCahnFunction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "<CahnFunction name=\"{}\" parameters={}>\n",
            self.func.name.fmt(&self.exec.string_data),
            self.func.param_count
        ))?;

        let code = &self.func.code;
        let code_map = &self.func.code_map;
        let num_consts = &self.exec.num_consts;
        let string_data = &self.exec.string_data;

        let mut code_reader = PanickingByteBufferReader::new(code);

        while !code_reader.is_at_end() {
            let start_index = code_reader.current_index();
            let code_pos = code_map[start_index];
            let instruction: Instruction = unsafe { mem::transmute(code_reader.read_u8()) };

            f.write_fmt(format_args!(
                "{}:{} \t{}\t{:?}",
                self.exec.source_file, code_pos, start_index, instruction
            ))?;

            match instruction {
                Instruction::LoadLitNum => {
                    f.write_fmt(format_args!("    '{}'", code_reader.read_u8()))?
                }

                Instruction::LoadConstNum => {
                    let index = code_reader.read_u8();
                    let val = self.exec.num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                }
                Instruction::LoadConstNumW => {
                    let index = code_reader.read_u16_le();
                    let val = num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                }
                Instruction::LoadConstNumWW => {
                    let index = code_reader.read_u32_le();
                    let val = num_consts[index as usize];
                    f.write_fmt(format_args!("    {} '{}'", index, val))?;
                }
                Instruction::JumpIfFalse | Instruction::Jump => {
                    let jump_location = code_reader.read_u32_le();
                    f.write_fmt(format_args!("    {}", jump_location))?;
                }

                Instruction::GetLocal | Instruction::SetLocal | Instruction::CreateListWithCap => {
                    f.write_fmt(format_args!("    {}", code_reader.read_u8()))?;
                }

                Instruction::LoadFunction => {
                    let func_index = code_reader.read_u32_le() as usize;
                    let func = &self.exec.functions[func_index];
                    f.write_fmt(format_args!(
                        "     {} '{}'",
                        func_index,
                        func.fmt(self.exec)
                    ))?;
                }

                Instruction::GetLocalW
                | Instruction::SetLocalW
                | Instruction::CreateListWithCapW => {
                    f.write_fmt(format_args!("    {}", code_reader.read_u16_le()))?;
                }

                Instruction::LoadStringLiteral => {
                    let start_index = code_reader.read_u32_le() as usize;
                    let end_index = code_reader.read_u32_le() as usize;

                    f.write_fmt(format_args!(
                        "    {}..{} '{}'",
                        start_index,
                        end_index,
                        &string_data[start_index..end_index]
                    ))?;
                }

                Instruction::CreateList => {}
                Instruction::ListPush => {}
                Instruction::Modulo => {}
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
                Instruction::ListGetIndex => {}
            }

            f.write_char('\n')?;
        }
        f.write_str("</CahnFunction>\n")?;
        Ok(())
    }
}
