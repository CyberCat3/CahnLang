use crate::compiler::lexical_analysis::Token;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum ParseError<'a> {
    #[error("bad token at index {}: {}: {}", .token.index, .token, .message)]
    BadToken {
        message: String,
        token: Token<'a>,
    },

    #[error("chaining comparison operators is not supported at index: {}: {}", .operator.index, .operator)]
    ChainingComparisonOperator {
        operator: Token<'a>
    },

    #[error("chaining assignment operators is not supported at index: {}: {}", .operator.index, .operator)]
    ChainingAssignmentOperator {
        operator: Token<'a>
    },
}

pub type Result<'a, T> = std::result::Result<T, ParseError<'a>>;