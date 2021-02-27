use cahn_lang::{
    compiler::{CodeGenerator, Parser},
    executable::Instruction,
};

#[test]
fn codegen() {
    let code = "print  2 +  2 * 3";

    let arena = bumpalo::Bump::new();
    let parser = Parser::from_str(code, &arena);
    let ast = parser.parse_program().unwrap();
    println!("ast: {}", ast);
    assert_eq!(ast.to_string(), "(program (print (+ 2 (* 2 3))))");

    let exec = CodeGenerator::new().gen(&ast);
    println!("exec: {}", exec);
    assert_eq!(exec.num_consts, vec![2.0, 3.0]);
    assert_eq!(
        exec.code,
        vec![
            Instruction::LoadNumber as u8,
            0,
            Instruction::LoadNumber as u8,
            0,
            Instruction::LoadNumber as u8,
            1,
            Instruction::Mul as u8,
            Instruction::Add as u8,
            Instruction::Print as u8,
        ]
    );
}
