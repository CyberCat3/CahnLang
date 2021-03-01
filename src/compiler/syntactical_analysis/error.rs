use crate::compiler::lexical_analysis::Token;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("bad token {}: {}", .token, .message)]
    BadToken { message: String, token: Token },

    #[error("unexpected token {}: {}", .token, .message)]
    UnexpectedToken { message: String, token: Token },

    #[error("chaining comparison operators is not supported: {}", .operator)]
    ChainingComparisonOperator { operator: Token },

    #[error("chaining assignment operators is not supported: {}", .operator)]
    ChainingAssignmentOperator { operator: Token },
}

pub type Result<'a, T> = std::result::Result<T, ParseError>;
