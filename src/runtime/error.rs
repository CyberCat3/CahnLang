#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError(TypeError),
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, RuntimeError>;