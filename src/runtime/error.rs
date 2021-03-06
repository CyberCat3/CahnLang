use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("TypeError: {}", .message)]
    TypeError { message: String },

    #[error("IndexOufOfBounds: attempted to element at index {}, but list only has length {}", .index, .len)]
    IndexOutOfBounds { index: f64, len: usize },

    #[error("couldn't write to stdout: {:?}", .0)]
    StdoutWriteError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
