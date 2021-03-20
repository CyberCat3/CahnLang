use std::iter;

use cahn_lang::{
    compiler::lexical_analysis::TokenPos,
    executable::{CahnFunction, Executable, Instruction},
    runtime::VM,
};

#[test]
fn function() {
    let code_map: Vec<_> = (0..1000).map(|_| TokenPos::new(1, 1)).collect();

    let main_code = vec![
        Instruction::LoadLitNum as u8,
        10,
        Instruction::LoadLitNum as u8,
        5,
        Instruction::GetLocal as u8,
        0,
        Instruction::GetLocal as u8,
        1,
        Instruction::LoadFunction as u8,
        1,
        Instruction::Invoke as u8,
        Instruction::Add as u8,
        Instruction::Print as u8,
        Instruction::Pop as u8,
        Instruction::Pop as u8,
    ];

    let add_code = vec![
        Instruction::GetLocal as u8,
        0,
        Instruction::GetLocal as u8,
        1,
        Instruction::Add as u8,
        Instruction::Return as u8,
        Instruction::LoadNil as u8,
        Instruction::Return as u8,
    ];

    let add_func = CahnFunction::new(2, add_code, code_map.clone(), 0, 3);

    let main_func = CahnFunction::new_anonymous(0, main_code, code_map);

    let num_consts = vec![];
    let string_data = "add".into();
    let source_file = "THE EATHERERER!".into();
    let functions = vec![main_func, add_func];
    let exec = Executable::new(num_consts, string_data, source_file, functions);

    VM::run_to_stdout(&exec).unwrap();
}
