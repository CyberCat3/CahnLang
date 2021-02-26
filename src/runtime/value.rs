use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
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