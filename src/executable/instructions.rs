#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Negate,
    Not,
    Add,
    Mul,
    Sub,
    Div,
    Modulo,
    Concat,

    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,

    CreateList,
    CreateListWithCap,
    CreateListWithCapW,
    ListPush,
    ListGetIndex,

    LoadTrue,
    LoadFalse,
    LoadNil,

    LoadStringLiteral,
    LoadLitNum,
    LoadConstNum,
    LoadConstNumW,
    LoadConstNumWW,

    SetLocal,
    SetLocalW,
    GetLocal,
    GetLocalW,

    LoadFunction,
    Invoke,
    Return,
    LoadReturnAdress,

    Dup,
    Pop,

    Print,

    Jump,
    JumpIfFalse,
}
