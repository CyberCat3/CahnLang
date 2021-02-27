#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Add,
    Mul,
    Sub,
    Div,
    Negate,

    Not,

    LoadNumber,
    LoadTrue,
    LoadFalse,
    LoadNil,

    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,

    Dup,
    Pop,
    
    Print,
}