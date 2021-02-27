use cahn_lang::{
    executable::{Executable, Instruction},
    runtime::{Value, VM},
};
#[test]
pub fn vm_test() {
    let exec = Executable {
        num_consts: vec![4.0, 5.0, 9.0],
        code: vec![
            // load 4 and 5 and add them together
            Instruction::LoadConstNum as u8,
            0,
            Instruction::LoadConstNum as u8,
            1,
            Instruction::Add as u8,
            // print the result
            Instruction::Dup as u8,
            Instruction::Print as u8,
            // load the expected result (9)
            Instruction::LoadConstNum as u8,
            2,
            // check if they are equal
            Instruction::Equal as u8,
        ],
    };

    let result = VM::new(&exec).run().unwrap();
    assert_eq!(result, vec![Value::Bool(true)]);
}
