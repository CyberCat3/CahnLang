use cahn_lang::{
    compiler::{CodeGenerator, Parser},
    runtime::{Value, VM},
};

#[test]
fn full_run_math() {
    let source = "(2 + 3) * -0.5 / 10 - -5";
    println!("source: {}", source);

    let arena = bumpalo::Bump::new();
    let parser = Parser::from_str(source, &arena);
    let ast = parser.parse_program().unwrap();
    println!("ast: {}", ast);

    assert_eq!(
        ast.to_string(),
        "(program (- (/ (* ((+ 2 3)) (- 0.5)) 10) (- 5)))"
    );

    let exec = CodeGenerator::new().gen(&ast);
    println!("exec: {:?}", exec);
    println!("{}", exec);

    let result = VM::new().run(&exec).unwrap();
    println!("result: {:?}", result);
    assert_eq!(result, vec![Value::Number(4.75)]);
}
