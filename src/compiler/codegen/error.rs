use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("unresolved variable at {}: {}", .index, .identifier)]
    UnresolvedVariable {
        identifier: String,
        index: usize,
    }
}

pub type Result<T> = std::result::Result<T, CodeGenError>;