use cahn_lang::compiler::{string_handling::StringInterner, syntactical_analysis::Parser};

#[test]
fn basic_precedence() {
    let src = "print 2 + 2 * 3";
    let arena = bumpalo::Bump::new();
    let interner = StringInterner::new();
    let parser = Parser::from_str(src, &arena, interner);
    let ast = parser.parse_program().unwrap();
    assert_eq!(&ast.to_string(), "(program (print (+ 2 (* 2 3)))\n)");
}

// pub fn parse_test() {
//     use crate::compiler::syntactical_analysis::Parser;
//     use bumpalo::Bump;

//     let source = "
//         if 2 = 2 then
//             print 5
//         elseif 2 = 3 then
//             print 8
//         else
//             print 7
//         end
//     ";

//     println!("source: {}", source);

//     let arena = Bump::new();

//     let parser = Parser::from_str(source, &arena);
//     let ast = parser.parse_program().unwrap();

//     println!("ast\n{}", ast);

//     println!("\nbytes allocated for ast: {}", arena.allocated_bytes());
// }
