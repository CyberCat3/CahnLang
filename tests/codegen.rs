use cahn_lang::{
    compiler::{string_handling::StringInterner, CodeGenerator, Parser},
    executable::Instruction,
};

#[test]
fn codegen() {
    let code = "print  2 +  2 * 3 + 0.3";

    let arena = bumpalo::Bump::new();
    let interner = StringInterner::new();
    let parser = Parser::from_str(code, &arena, interner);
    let ast = parser.parse_program().unwrap();
    println!("ast: {}", ast);
    assert_eq!(ast.to_string(), "(program (print (+ (+ 2 (* 2 3)) 0.3)))");

    let exec = CodeGenerator::new().gen(&ast).unwrap();
    println!("exec: {}", exec);
    assert_eq!(exec.num_consts, vec![0.3]);
    assert_eq!(
        exec.code,
        vec![
            Instruction::LoadLitNum as u8,
            2,
            Instruction::LoadLitNum as u8,
            2,
            Instruction::LoadLitNum as u8,
            3,
            Instruction::Mul as u8,
            Instruction::Add as u8,
            Instruction::LoadConstNum as u8,
            0,
            Instruction::Add as u8,
            Instruction::Print as u8,
            Instruction::LoadNil as u8,
        ]
    );
}
