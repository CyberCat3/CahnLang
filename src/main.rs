use std::fs;

use cahn_lang::{compiler::{CodeGenerator, Parser}, runtime::VM};

fn main() {
    let src = fs::read_to_string("./test.cahn").unwrap();
    println!("SOURCE CODE\n{}", src);

    let arena = bumpalo::Bump::new();
    
    let ast = Parser::from_str(&src, &arena).parse_program().unwrap();

    println!("\nAST ({} bytes allocated)\n{}", arena.allocated_bytes(), ast);
    
    let exec = CodeGenerator::new().gen(&ast);

    println!("{}", exec);
    println!("raw code: {:?}", exec.code);

    let results = VM::new().run(&exec).unwrap();
    println!("results: {:?}", results);
}
