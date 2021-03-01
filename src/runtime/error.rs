use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("TypeError: {}", .message)]
    TypeError { message: String },

    #[error("couldn't write to stdout: {:?}", .0)]
    StdoutWriteError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
