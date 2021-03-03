pub mod compiler;
pub mod executable;
pub mod runtime;

use std::hash::Hasher;

use ahash::AHasher;
use compiler::{string_handling::StringInterner, CodeGenerator, Parser};
use runtime::VM;

pub fn execute_source_to_string(source: &str, file_name: String) -> String {
    let interner = StringInterner::new();
    let arena = bumpalo::Bump::new();

    let ast = Parser::from_str(source, &arena, interner.clone())
        .parse_program()
        .unwrap();

    let exec = CodeGenerator::new(interner, file_name).gen(&ast).unwrap();

    VM::run_to_string(&exec).unwrap()
}

pub fn hash_string(string: &str) -> u64 {
    let mut hasher = AHasher::default();
    hasher.write(string.as_bytes());
    hasher.finish()
}
