use crate::{
    executable::{Executable, Instruction},
    runtime::{
        error::{Result, RuntimeError},
        mem_manager::MemoryManager,
        Value,
    },
};

use std::{
    cell::RefCell,
    fmt::{self, Debug},
    io::{self, Write},
    mem,
};

pub struct VM<'a> {
    pub stack: Vec<Value>,
    ip: usize,
    fp: usize,
    pub exec: &'a Executable,
    mem_manager: RefCell<MemoryManager>,

    stdout: RefCell<&'a mut dyn Write>,
}

impl<'a> Debug for VM<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "VM(ip: {}, fp: {}, stack: {:?})",
            self.ip, self.fp, self.stack
        ))
    }
}

impl<'a> VM<'a> {
    pub fn new(exec: &'a Executable, stdout: &'a mut dyn Write) -> Self {
        VM {
            stdout: RefCell::new(stdout),
            stack: Vec::new(),
            ip: 0,
            fp: 0,
            exec,
            mem_manager: RefCell::new(MemoryManager::new()),
        }
    }

    pub fn run_to_stdout(exec: &'a Executable) -> Result<()> {
        let mut stdout = io::stdout();
        let vm = VM::new(exec, &mut stdout);
        vm.run()
    }

    pub fn run_to_string(exec: &'a Executable) -> Result<String> {
        let mut bytes: Vec<u8> = vec![];
        let vm = VM::new(exec, &mut bytes);
        vm.run()?;
        Ok(String::from_utf8(bytes).expect("VM shouldn't be able to produce invalid utf8"))
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
        let val = u16::from_le_bytes([code[self.ip], code[self.ip + 1]]);
        self.ip += 2;
        val
    }

    #[inline]
    fn read_u32(&mut self) -> u32 {
        let code = &self.exec.code;
        let val = u32::from_le_bytes([
            code[self.ip],
            code[self.ip + 1],
            code[self.ip + 2],
            code[self.ip + 3],
        ]);
        self.ip += 4;
        val
    }

    #[inline]
    fn exec_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::LoadStringLiteral => {
                let start_index = self.read_u32();
                let end_index = self.read_u32();
                self.push(Value::StringLiteral {
                    start_index,
                    end_index,
                });
            }

            Instruction::Concat => {
                let right_val = self.pop();
                let left_val = self.pop();
                let new_string = format!("{}{}", left_val.fmt(&self), right_val.fmt(&self));

                let new_val = self
                    .mem_manager
                    .borrow_mut()
                    .alloc_string(&self, new_string);

                self.push(new_val);
            }

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
                                left.fmt(self),
                                right.fmt(self)
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
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "subtract-instruction expected two numbers, but got '{}' and '{}'",
                                left.fmt(self),
                                right.fmt(self)
                            ),
                        })
                    }
                }
            }

            Instruction::Mul => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Number(left_num * right_val)),
                    _ => return Err(RuntimeError::TypeError {message: format!("multiplication-instruction expected two numbers, but got '{}' and '{}'", left.fmt(self), right.fmt(self))}),
                }
            }

            Instruction::Div => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Number(left_num / right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "division-instruction expected two numbers, but got '{}' and '{}'",
                                left.fmt(self),
                                right.fmt(self)
                            ),
                        })
                    }
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
                                val.fmt(self)
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
                                left.fmt(self),
                                right.fmt(self)
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
                                left.fmt(self),
                                right.fmt(self)
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
                                left.fmt(self),
                                right.fmt(self)
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
                                left.fmt(self),
                                right.fmt(self)
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
                let val = self.pop();
                // let out = mem::replace(self.stdout);
                write!(self.stdout.borrow_mut(), "{}\n", val.fmt(self))?;
            }

            Instruction::Jump => {
                let jump_location = self.read_u32() as usize;
                self.ip = jump_location;
            }

            Instruction::JumpIfFalse => {
                let jump_location = self.read_u32() as usize;
                if !self.pop().is_truthy() {
                    self.ip = jump_location;
                }
            }
        };
        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        while self.ip < self.exec.code.len() {
            let instruction = self.read_instruction();
            // println!("about to run: {:?}", instruction);

            // let mut string = String::new();
            // std::io::stdin().read_line(&mut string).unwrap();

            self.exec_instruction(instruction)?;

            // let mut padding = String::new();
            // let ins_str = format!("{:?}", instruction);

            // for _ in 0..(20 - ins_str.len()) {
            //     padding.push('-');
            // }

            // print!("{:?}{}-->   ", instruction, padding);
            // for val in &self.stack {
            //     print!("{:?}({})   ", val, (*val).fmt(&self));
            // }
            // println!();
        }
        Ok(())
    }
}
