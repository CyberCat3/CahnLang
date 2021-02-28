use thiserror::Error;

use crate::compiler::string_handling::StringAtom;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("unresolved variable at {}: {}", .index, .identifier)]
    UnresolvedVariable {
        identifier: StringAtom,
        index: usize,
    },
}

pub type Result<T> = std::result::Result<T, CodeGenError>;
