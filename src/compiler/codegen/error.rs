use thiserror::Error;

use crate::compiler::lexical_analysis::Token;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("unresolved variable at {}: {}", .var_token.pos, .var_token.lexeme)]
    UnresolvedVariable { var_token: Token },

    #[error("invalid assignment target: {}", .message)]
    // todo there should be an ast node included in this
    InvalidAssignmentTarget { message: String },
}

pub type Result<T> = std::result::Result<T, CodeGenError>;
