pub mod compiler;
pub mod runtime;
pub mod executable;

pub fn parse_test() {
    use crate::compiler::syntactical_analysis::Parser;
    use bumpalo::Bump;
    
    let source = "
        if 2 = 2 then
            print 5
        elseif 2 = 3 then
            print 8
        else
            print 7
        end
    ";

    println!("source: {}", source);

    let arena = Bump::new();

    let parser = Parser::from_str(source, &arena);
    let ast = parser.parse_program().unwrap();

    println!("ast\n{}", ast);

    println!("\nbytes allocated for ast: {}", arena.allocated_bytes());
}

pub fn exec_test() {
    use crate::{runtime::VM, executable::{Instruction, Executable}};

    println!("size of executable: {}", std::mem::size_of::<Executable>());

    let executable = Executable {
        num_consts: vec![4.0, 5.0, 9.0],
        code: vec![
            // load 4 and 5 and add them together
            Instruction::LoadNumber as u8, 0,
            Instruction::LoadNumber as u8, 1,
            Instruction::Add as u8,

            // print the result
            Instruction::Dup as u8,
            Instruction::Print as u8,

            // load the expected result (9)
            Instruction::LoadNumber as u8, 2,

            // check if they are equal
            Instruction::Equal as u8,

            // print whether or not they were
            Instruction::Print as u8,
        ]
    };

    let mut vm = VM::new(executable);
    vm.run().unwrap();
}
 


