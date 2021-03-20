mod function;
mod instructions;

pub use function::CahnFunction;
pub use instructions::Instruction;

use std::fmt;

#[derive(Clone)]
pub struct Executable {
    pub num_consts: Vec<f64>,

    pub functions: Vec<CahnFunction>,

    pub source_file: String,
    pub string_data: String,
}

impl Executable {
    pub fn new(
        num_consts: Vec<f64>,

        string_data: String,

        source_file: String,

        functions: Vec<CahnFunction>,
    ) -> Self {
        Executable {
            string_data,
            source_file,
            num_consts,
            functions,
        }
    }
}

impl fmt::Debug for Executable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "\n<CahnExecutable>
NUM_CONSTS: {:?}

STRING_DATA: '{}'
    
FUNCTIONS\n",
            self.num_consts, self.string_data,
        ))?;

        for func in &self.functions {
            fmt::Debug::fmt(&func.fmt(&self), f)?;
        }

        f.write_str("</CahnExecutable>\n")?;

        Ok(())
    }
}
