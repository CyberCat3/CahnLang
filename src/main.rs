use ahash::AHashMap;
use string_interner::StringInterner;


fn main() {
    use std::fs;

    use cahn_lang::{compiler::{CodeGenerator, Parser}, runtime::VM};
    let src = fs::read_to_string("./test.cahn").unwrap();
    println!("SOURCE CODE\n{}", src);

    let arena = bumpalo::Bump::new();
    
    let ast = Parser::from_str(&src, &arena).parse_program().unwrap();

    println!("\nAST ({} bytes allocated)\n{}", arena.allocated_bytes(), ast);
    
    let exec = CodeGenerator::new().gen(&ast).unwrap();

    println!("{}", exec);
    println!("raw code: {:?}", exec.code);

    println!("\n<VM STDOUT>");
    let results = VM::new(&exec).run().unwrap();
    println!("</VM STDOUT>");
    println!("results: {:?}", results);
}
