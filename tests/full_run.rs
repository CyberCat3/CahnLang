use cahn_lang::{
    compiler::{string_handling::StringInterner, CodeGenerator, Parser},
    runtime::{Value, VM},
};

#[test]
fn full_run_math() {
    let source = "(2 + 3) * -0.5 / 10 - -5";
    println!("source: {}", source);

    let arena = bumpalo::Bump::new();
    let interner = StringInterner::new();
    let parser = Parser::from_str(source, &arena, interner);
    let ast = parser.parse_program().unwrap();
    println!("ast: {}", ast);

    assert_eq!(
        ast.to_string(),
        "(program (- (/ (* ((+ 2 3)) (- 0.5)) 10) (- 5)))"
    );

    let exec = CodeGenerator::new().gen(&ast).unwrap();
    println!("exec: {:?}", exec);
    println!("{}", exec);

    let result = VM::new(&exec).run().unwrap();
    println!("result: {:?}", result);
    assert_eq!(result, vec![Value::Number(4.75)]);
}

#[test]
fn full_run_add_and_block() {
    let source = "
    
    10 + 8 + 20 + 700 + 0.1 + 0.1 +    
        block
            print 3
            print 4
            print 5
            7
        end
";
    println!("source: {}", source);

    let arena = bumpalo::Bump::new();
    let interner = StringInterner::new();
    let parser = Parser::from_str(source, &arena, interner);
    let ast = parser.parse_program().unwrap();

    let exec = CodeGenerator::new().gen(&ast).unwrap();
    println!("exec: {:?}", exec);
    println!("{}", exec);

    assert_eq!(exec.num_consts, vec![700.0, 0.1]);

    let result = VM::new(&exec).run().unwrap();
    println!("result: {:?}", result);
    assert_eq!(result, vec![Value::Number(745.2)]);
}
