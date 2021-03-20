use std::fmt;

use crate::compiler::string_handling::StringAtom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleDot,
    DoubleStar,
    DoubleSlash,

    Identifier,
    Number,
    String,
    True,
    False,
    Nil,

    ParenOpen,
    ParenClose,

    BracketOpen,
    BracketClose,

    BraceOpen,
    BraceClose,

    Let,

    Comma,

    BangEqual,
    DoubleEqual,
    ColonEqual,

    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Fn,
    Return,

    If,
    Else,
    While,

    And,
    Or,
    Not,

    Print,

    Eof,
    Semicolon,
    BadCharacter,
}

pub mod token_groups {
    use super::TokenType::{self, *};

    pub const BLOCK_ENDINGS: &[TokenType] = &[BraceClose, Eof];

    pub const LITERALS: &[TokenType] = &[Number, True, False];
    pub const COMPARISON_OPERATORS: &[TokenType] = &[
        DoubleEqual,
        Less,
        LessEqual,
        Greater,
        GreaterEqual,
        BangEqual,
    ];
    pub const PREFIX_OPERATORS: &[TokenType] = &[Not, Minus];
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPos {
    pub line: usize,
    pub column: usize,
}

impl TokenPos {
    pub fn new(line: usize, column: usize) -> Self {
        TokenPos { line, column }
    }
}

impl fmt::Display for TokenPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.line, self.column))
    }
}

impl Default for TokenPos {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub pos: TokenPos,
    pub token_type: TokenType,
    pub lexeme: StringAtom,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "[{}]{}(\"{}\")",
            self.pos, self.token_type, self.lexeme
        ))
    }
}
