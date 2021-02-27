use crate::{
    executable::{Executable, Instruction},
    runtime::{
        error::{Result, RuntimeError},
        Value,
    },
};

use std::{io::stdin, mem};

#[derive(Debug, Clone)]
pub struct VM<'a> {
    stack: Vec<Value>,
    ip: usize,
    fp: usize,
    exec: &'a Executable,
}

impl<'a> VM<'a> {
    pub fn new(exec: &'a Executable) -> Self {
        VM { stack: Vec::new(), ip: 0, fp: 0, exec }
    }

    #[inline]
    fn peek(&mut self) -> Value {
        *self.stack.last().unwrap()
    }

    #[inline]
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    #[inline]
    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    #[inline]
    fn read_u8(&mut self) -> u8 {
        let byte = self.exec.code[self.ip];
        self.ip += 1;
        byte
    }

    #[inline]
    fn read_instruction(&mut self) -> Instruction {
        let byte = self.read_u8();
        unsafe { mem::transmute(byte) }
    }

    #[inline]
    fn read_u16(&mut self) -> u16 {
        let code = &self.exec.code;
        let val = u16::from_le_bytes([
            code[self.ip],
            code[self.ip+1]
        ]);
        self.ip += 2;
        val
    }

    #[inline]
    fn read_u32(&mut self) -> u32 {
        let code = &self.exec.code;
        let val = u32::from_le_bytes([
            code[self.ip],
            code[self.ip+1],
            code[self.ip+2],
            code[self.ip+3]
        ]);
        self.ip += 4;
        val
    }

    #[inline]
    fn exec_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::LoadConstNum => {
                let num_index = self.read_u8();
                self.push(Value::Number(self.exec.num_consts[num_index as usize]));
            }

            Instruction::LoadConstNumW => {
                let num_index = self.read_u16();
                self.push(Value::Number(self.exec.num_consts[num_index as usize]));
            }

            Instruction::LoadConstNumWW => {
                let num_index = self.read_u32();
                self.push(Value::Number(self.exec.num_consts[num_index as usize]));
            }

            Instruction::LoadLitNum => {
                let num = self.read_u8();
                self.push(Value::Number(num as f64));
            }

            Instruction::SetLocal => {
                let stack_offset = self.read_u8();
                self.stack[self.fp + stack_offset as usize] = self.pop();
            }

            Instruction::SetLocalW => {
                let stack_offset = self.read_u16();
                self.stack[self.fp + stack_offset as usize] = self.pop();
            }

            Instruction::GetLocal => {
                let stack_offset = self.read_u8();
                self.push(self.stack[self.fp + stack_offset as usize]);
            }

            Instruction::GetLocalW => {
                let stack_offset = self.read_u16();
                self.push(self.stack[self.fp + stack_offset as usize]);
            }

            Instruction::LoadTrue => self.push(Value::Bool(true)),
            Instruction::LoadFalse => self.push(Value::Bool(false)),
            Instruction::LoadNil => self.push(Value::Nil),

            Instruction::Add => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Number(left_num + right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "add-instruction expected two numbers, but got '{}' and '{}'",
                                left, right
                            ),
                        })
                    }
                }
            }

            Instruction::Sub => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Number(left_num - right_val))
                    }
                    _ => return Err(RuntimeError::TypeError {
                        message: format!(
                            "subtract-instruction expected two numbers, but got '{}' and '{}'",
                            left, right
                        ),
                    }),
                }
            }

            Instruction::Mul => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Number(left_num * right_val)),
                    _ => return Err(RuntimeError::TypeError {message: format!("multiplication-instruction expected two numbers, but got '{}' and '{}'", left, right)}),
                }
            }

            Instruction::Div => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Number(left_num / right_val))
                    }
                    _ => return Err(RuntimeError::TypeError {
                        message: format!(
                            "division-instruction expected two numbers, but got '{}' and '{}'",
                            left, right
                        ),
                    }),
                }
            }

            Instruction::Negate => {
                let val = self.pop();

                match val {
                    Value::Number(num) => self.push(Value::Number(-num)),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "negate-instruction expected a number, but got '{}'",
                                val
                            ),
                        })
                    }
                };
            }

            Instruction::Not => {
                let val = self.pop();
                self.push(Value::Bool(!val.is_truthy()));
            }

            Instruction::LessThan => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Bool(left_num < right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "'<' operator expected two numbers, but got '{}' and '{}'",
                                left, right
                            ),
                        })
                    }
                }
            }

            Instruction::LessThanOrEqual => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Bool(left_num <= right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "'<=' operator expected two numbers, but got '{}' and '{}'",
                                left, right
                            ),
                        })
                    }
                }
            }

            Instruction::GreaterThan => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Bool(left_num > right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "'>' operator expected two numbers, but got '{}' and '{}'",
                                left, right
                            ),
                        })
                    }
                }
            }

            Instruction::GreaterThanOrEqual => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Bool(left_num >= right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "'>=' operator expected two numbers, but got '{}' and '{}'",
                                left, right
                            ),
                        })
                    }
                }
            }

            Instruction::Equal => {
                let right = self.pop();
                let left = self.pop();

                self.push(Value::Bool(left == right));
            }

            Instruction::Dup => {
                let val = self.peek();
                self.push(val);
            }

            Instruction::Pop => {
                self.pop();
            }

            Instruction::Print => {
                println!("{}", self.pop());
            }
        };

        Ok(())
    }

    pub fn run(mut self) -> Result<Vec<Value>> {
        while self.ip < self.exec.code.len() {
            let instruction = self.read_instruction();
            // println!("about to run: {:?}", instruction);

            // let mut string = String::new();
            // stdin().read_line(&mut string).unwrap();

            self.exec_instruction(instruction)?;

            // println!("stack is now: {:?}", self.stack);
        }
        Ok(self.stack)
    }
}
