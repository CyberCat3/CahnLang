pub mod instructions;

pub use instructions::Instruction as Instruction;

#[derive(Debug, Clone)]
pub struct Executable {
    pub num_consts: Vec<f64>,
    pub code: Vec<u8>,
}