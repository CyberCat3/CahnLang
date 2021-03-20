use crate::{
    executable::{CahnFunction, Executable, Instruction},
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

use super::mem_manager::HeapValue;

pub struct VM<'a> {
    pub exec: &'a Executable,
    mem_manager: RefCell<MemoryManager>,

    pub stack: Vec<Value>,

    pub curr_func: &'a CahnFunction,
    ip: usize,
    fp: usize,

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
            mem_manager: RefCell::new(MemoryManager::new()),
            exec,

            stack: Vec::new(),

            curr_func: exec
                .functions
                .last()
                .expect("CodeGenerator didn't create any functions ¯\\_(ツ)_/¯"),

            ip: 0,
            fp: 0,

            stdout: RefCell::new(stdout),
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
        let byte = self.curr_func.code[self.ip];
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
        let code = &self.curr_func.code;
        let val = u16::from_le_bytes([code[self.ip], code[self.ip + 1]]);
        self.ip += 2;
        val
    }

    #[inline]
    fn read_u32(&mut self) -> u32 {
        let code = &self.curr_func.code;
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
    fn read_u64(&mut self) -> u64 {
        let code = &self.curr_func.code;
        let val = u64::from_le_bytes([
            code[self.ip],
            code[self.ip + 1],
            code[self.ip + 2],
            code[self.ip + 3],
            code[self.ip + 4],
            code[self.ip + 5],
            code[self.ip + 6],
            code[self.ip + 7],
        ]);
        self.ip += 8;
        val
    }

    #[inline]
    fn get_local(&self, stack_offset: usize) -> Value {
        self.stack[self.fp + stack_offset]
    }

    fn assert_function<'b>(&'b self, val: Value) -> &'a CahnFunction {
        match val {
            Value::Function { function_index } => &self.exec.functions[function_index as usize],
            other => panic!(
                "invalid bytecode, expected value to be a function, got: {}",
                other.fmt(&self)
            ),
        }
    }

    fn assert_return_address(&self, val: Value) -> usize {
        match val {
            Value::ReturnAdress { ip } => ip,
            other => panic!(
                "invalid bytecode, expected value to be a return adress, got: {}",
                other.fmt(&self)
            ),
        }
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
                self.push(self.get_local(stack_offset as usize))
            }

            Instruction::GetLocalW => {
                let stack_offset = self.read_u16();
                self.push(self.get_local(stack_offset as usize))
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

            Instruction::Modulo => {
                let right = self.pop();
                let left = self.pop();

                match (left, right) {
                    (Value::Number(left_num), Value::Number(right_val)) => {
                        self.push(Value::Number(left_num % right_val))
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "modulo-instruction expected two numbers, but got '{}' and '{}'",
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
            Instruction::CreateList => {
                let list = self.mem_manager.borrow_mut().alloc_list(self, 0);
                self.push(list)
            }
            Instruction::CreateListWithCap => {
                let init_cap = self.read_u8() as usize;
                let list = self.mem_manager.borrow_mut().alloc_list(self, init_cap);
                self.push(list)
            }
            Instruction::CreateListWithCapW => {
                let init_cap = self.read_u16() as usize;
                let list = self.mem_manager.borrow_mut().alloc_list(self, init_cap);
                self.push(list)
            }
            Instruction::ListPush => {
                let right = self.pop();
                let list_val = self.peek();

                (|| unsafe {
                    if let Value::Heap(ptr) = list_val {
                        if let HeapValue::List(list) = &mut (*ptr).payload {
                            list.push(right);
                            return Ok(());
                        }
                    }
                    return Err(RuntimeError::TypeError {
                        message: format!(
                            "tried to push an element to a non-list type: '{}'",
                            right.fmt(self)
                        ),
                    });
                })()?;
            }

            Instruction::ListGetIndex => {
                let index = self.pop();
                let list = self.pop();

                let list = (|| unsafe {
                    if let Value::Heap(ptr) = list {
                        if let HeapValue::List(list) = &mut (*ptr).payload {
                            return Ok(list);
                        }
                    }
                    Err(RuntimeError::TypeError {
                        message: format!("[] operator expected a list, got {}", list.fmt(self)),
                    })
                })()?;

                let index = match index {
                    Value::Number(num) => {
                        if num < 0.0 || num as usize >= list.len() {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: num,
                                len: list.len(),
                            });
                        }
                        num as usize
                    }

                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!(
                                "[] operator expected number, got {}",
                                index.fmt(self)
                            ),
                        })
                    }
                };

                self.push(list[index]);
            }

            Instruction::LoadFunction => {
                let function_index = self.read_u32();
                self.push(Value::Function { function_index })
            }

            Instruction::Invoke => {
                let arg_count = self.read_u8();
                self.fp = self.stack.len() - arg_count as usize - 1;

                let func = match self.get_local(0) {
                    Value::Function { function_index } => {
                        &self.exec.functions[function_index as usize]
                    }
                    // todo better error message here
                    other => {
                        return Err(RuntimeError::TypeError {
                            message: format!("attempted to call a non-function: {:?}", other),
                        })
                    }
                };

                self.curr_func = func;
                self.ip = 0;
            }

            Instruction::Return => {
                println!("return start");
                self.print_stack();
                let return_val = self.pop();
                println!("return val: {}", return_val.fmt(self));
                let called_function = self.assert_function(self.get_local(0));

                println!("called_function: {}", called_function.fmt(&self.exec));
                for _ in 0..called_function.param_count {
                    self.pop(); // pop all arguments
                }
                println!("Popped args");
                self.print_stack();
                self.pop(); // pop called function
                println!("popped called function");
                self.print_stack();

                // return to the previous function
                let stack_top = self.pop();
                let callee_function = self.assert_function(stack_top);
                self.curr_func = callee_function;

                // set the instruction pointer to the return address
                let stack_top = self.pop();
                let return_address = self.assert_return_address(stack_top);
                self.ip = return_address;

                // push the return value
                self.push(return_val);
            }

            Instruction::LoadReturnAdress => {
                let ret_addr = self.read_u64() as usize;
                self.push(Value::ReturnAdress { ip: ret_addr });
            }
        };
        Ok(())
    }

    fn print_stack(&self) {
        for (index, val) in self.stack.iter().enumerate() {
            if index == self.fp {
                print!("<fp>");
            }
            print!("{}   ", (*val).fmt(&self));
        }
        println!();
    }

    pub fn run(mut self) -> Result<()> {
        while self.ip < self.curr_func.code.len() {
            let instruction = self.read_instruction();
            // println!("about to run: {:?}", instruction);

            // let mut string = String::new();
            // std::io::stdin().read_line(&mut string).unwrap();

            let code_pos = self.curr_func.code_map[self.ip];

            self.exec_instruction(instruction)?;

            let mut padding = String::new();
            let ins_str = format!("{:?}", instruction);

            for _ in 0..(20 - ins_str.len()) {
                padding.push('-');
            }

            print!(
                "{}:{}\t{:?}{}-->   ",
                self.exec.source_file, code_pos, instruction, padding,
            );

            self.print_stack();
        }
        Ok(())
    }
}
