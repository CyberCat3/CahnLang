use std::{fmt, error};
use crate::compiler::lexical_analysis::Token;

#[derive(Debug, Clone)]
pub enum ParseError<'a> {
    BadToken(BadTokenError<'a>),
    ChainingComparisonOperator(ChainingComparatorOperatorError<'a>),
    ChainingAssignmentOperator(ChainingAssignmentOperatorError<'a>),
}


impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("syntax error: ")?;
        match self {
            ParseError::BadToken(bte) => f.write_fmt(format_args!(
                "bad token at index {}: {}: {}",
                bte.token.index, bte.token, bte.message
            ))?,

            ParseError::ChainingComparisonOperator(ccoe) => f.write_fmt(format_args!(
                "chaining comparison operators is not supported at index: {}: {}",
                ccoe.operator_token.index, ccoe.operator_token
            ))?,

            ParseError::ChainingAssignmentOperator(caoe) => f.write_fmt(format_args!(
                "chaining assignment operators is not supported at index: {}: {}",
                caoe.operator_token.index, caoe.operator_token
            ))?,
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BadTokenError<'a> {
    pub message: String,
    pub token: Token<'a>,
}

#[derive(Debug, Clone)]
pub struct ChainingComparatorOperatorError<'a> {
    pub operator_token: Token<'a>,
}

#[derive(Debug, Clone)]
pub struct ChainingAssignmentOperatorError<'a> {
    pub operator_token: Token<'a>,
}

pub type Result<'a, T> = std::result::Result<T, ParseError<'a>>;