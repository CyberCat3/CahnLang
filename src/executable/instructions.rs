#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Add,
    Mul,
    Sub,
    Div,
    Negate,

    Not,

    LoadConstNum,
    LoadConstNumW,
    LoadConstNumWW,

    SetLocal,
    SetLocalW,
    GetLocal,
    GetLocalW,

    LoadLitNum,
    LoadStringLiteral,

    Concat,

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

    Jump,
    JumpIfFalse,
}
