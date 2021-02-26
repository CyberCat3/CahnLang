use cahn_lang::compiler::syntactical_analysis::Parser;

#[test]
fn basic_precedence() {
    let src = "2 + 2 * 3";
    let arena = bumpalo::Bump::new();
    let parser = Parser::from_str(src, &arena);
    let ast = parser.parse_program().unwrap();
    assert_eq!(&ast.to_string(), "(program (+ 2 (* 2 3)))");
}