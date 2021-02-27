use crate::{
    executable::{Executable, Instruction},
    runtime::{
        Value,
        error::{
            RuntimeError,
            Result
        }
    }
};

use std::mem;


#[derive(Debug, Clone)]
pub struct VM {
    exec: Executable,
    stack: Vec<Value>,
}

impl VM {
    pub fn new(exec: Executable) -> Self {
        VM {
            exec,
            stack: Vec::new(),
        }
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

    pub fn run(&mut self) -> Result<()> {
        let mut ip = 0;
        while ip < self.exec.code.len() {
            let instruction: Instruction = unsafe { mem::transmute(self.exec.code[ip]) };

            ip += 1;

            match instruction {
                Instruction::LoadNumber => {
                    let num_index = self.exec.code[ip];
                    ip += 1;
                    self.push(Value::Number(self.exec.num_consts[num_index as usize]));  
                },

                Instruction::LoadTrue => self.push(Value::Bool(true)),
                Instruction::LoadFalse => self.push(Value::Bool(false)),
                Instruction::LoadNil => self.push(Value::Nil),

                Instruction::Add => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => {
                            self.push(Value::Number(left_num + right_val));
                        },
                        _ => {
                            return Err(RuntimeError::TypeError {message: format!("add-instruction expected two numbers, but got '{}' and '{}'", left, right)});
                        }
                    }
                },
                
                Instruction::Sub => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => {
                            self.push(Value::Number(left_num - right_val));
                        },
                        _ => {
                            return Err(RuntimeError::TypeError {message: format!("subtract-instruction expected two numbers, but got '{}' and '{}'", left, right)});
                        }
                    }
                },

                Instruction::Mul => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => {
                            self.push(Value::Number(left_num * right_val));
                        },
                        _ => {
                            return Err(RuntimeError::TypeError {message: format!("multiplication-instruction expected two numbers, but got '{}' and '{}'", left, right)});
                        }
                    }
                },

                Instruction::Div => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Number(left_num / right_val)),
                        _ => return Err(RuntimeError::TypeError {message: format!("division-instruction expected two numbers, but got '{}' and '{}'", left, right)}),
                    }
                },
                
                Instruction::Negate => {
                    let val = self.pop();

                    match val {
                        Value::Number(num) => self.push(Value::Number(-num)),
                        _ => return Err(RuntimeError::TypeError {message: format!("negate-instruction expected a number, but got '{}'", val)})
                    };
                },
                
                Instruction::Not => {
                    let val = self.pop();
                    self.push(Value::Bool(!val.is_truthy()));
                },

                Instruction::LessThan => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Bool(left_num < right_val)),
                        _ => return Err(RuntimeError::TypeError {message: format!("'<' operator expected two numbers, but got '{}' and '{}'", left, right)}),
                    }
                },

                Instruction::LessThanOrEqual => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Bool(left_num <= right_val)),
                        _ => return Err(RuntimeError::TypeError {message: format!("'<=' operator expected two numbers, but got '{}' and '{}'", left, right)}),
                    }
                },

                Instruction::GreaterThan => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Bool(left_num > right_val)),
                        _ => return Err(RuntimeError::TypeError {message: format!("'>' operator expected two numbers, but got '{}' and '{}'", left, right)}),
                    }
                },

                Instruction::GreaterThanOrEqual => {
                    let right = self.pop();
                    let left = self.pop();

                    match (left, right) {
                        (Value::Number(left_num), Value::Number(right_val)) => self.push(Value::Bool(left_num >= right_val)),
                        _ => return Err(RuntimeError::TypeError {message: format!("'>=' operator expected two numbers, but got '{}' and '{}'", left, right)}),
                    }
                },
                
                Instruction::Equal => {
                    let right = self.pop();
                    let left = self.pop();

                    self.push(Value::Bool(left == right));
                },
                
                Instruction::Dup => {
                    let val = self.peek();
                    self.push(val);
                },

                Instruction::Pop => { self.pop(); },
                
                Instruction::Print => { println!("{}", self.pop()); },
            };
        }
        Ok(())
    }
}