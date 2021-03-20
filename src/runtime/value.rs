use std::fmt::{self, Write};

use super::{mem_manager::HeapValueHeader, VM};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    StringLiteral { start_index: u32, end_index: u32 },
    Heap(*mut HeapValueHeader),
    Function { function_index: u32 },
    ReturnAdress { ip: usize },
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(match self {
            Value::Nil => f.write_str("Nil")?,
            Value::Bool(b) => f.write_fmt(format_args!("Bool({})", b))?,
            Value::Number(num) => f.write_fmt(format_args!("Number({})", num))?,

            Value::StringLiteral {
                start_index,
                end_index,
            } => f.write_fmt(format_args!(
                "StringLiteral({}..{})",
                start_index, end_index
            ))?,

            Value::Function { function_index } => {
                f.write_fmt(format_args!("Format(index: {})", function_index))?
            }

            Value::ReturnAdress { ip } => f.write_fmt(format_args!("ReturnAdress({})", ip))?,

            Value::Heap(ptr) => f.write_fmt(format_args!("HeapPtr({:?})", *ptr))?,
        })
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }

    pub fn fmt<'a, 'b>(self, vm: &'a VM<'b>) -> FormatableValue<'a, 'b> {
        FormatableValue { value: self, vm }
    }
}

pub struct FormatableValue<'a, 'b> {
    value: Value,
    vm: &'a VM<'b>,
}

impl<'a, 'b> fmt::Display for FormatableValue<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Value::Bool(b) => f.write_fmt(format_args!("{}", b)),
            Value::Nil => f.write_str("nil"),
            Value::Number(num) => f.write_fmt(format_args!("{}", num)),

            Value::Function { function_index } => {
                let cahn_function = &self.vm.exec.functions[function_index as usize];
                let cahn_function = cahn_function.fmt(&self.vm.exec);
                fmt::Display::fmt(&cahn_function, f)
            }

            Value::ReturnAdress { ip } => f.write_fmt(format_args!("<returnaddr {}>", ip)),

            Value::StringLiteral {
                start_index,
                end_index,
            } => f.write_str(&self.vm.exec.string_data[start_index as usize..end_index as usize]),

            Value::Heap(heap_val) => unsafe { fmt::Display::fmt(&(*heap_val).fmt(self.vm), f) },
        }
    }
}
