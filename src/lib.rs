pub mod compiler;
pub mod executable;
pub mod runtime;
pub mod utils;

use compiler::{string_handling::StringInterner, CodeGenerator, Parser};
use runtime::VM;

pub fn execute_source_to_string(source: &str, file_name: String) -> String {
    let interner = StringInterner::new();
    let arena = bumpalo::Bump::new();

    let ast = Parser::from_str(source, &arena, interner)
        .parse_program()
        .unwrap();

    let exec = CodeGenerator::gen_executable(file_name, &ast).unwrap();

    VM::run_to_string(&exec).unwrap()
}
