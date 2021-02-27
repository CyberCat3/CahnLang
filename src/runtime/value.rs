use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            _ => true
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => f.write_str("nil")?,
            Value::Bool(bool) => fmt::Display::fmt(bool, f)?,
            Value::Number(num) => fmt::Display::fmt(num, f)?,
        }
        Ok(())
    }
}