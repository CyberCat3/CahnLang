use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    
    #[error("TypeError: {}", .message)]
    TypeError {
        message: String,
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;