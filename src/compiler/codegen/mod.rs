use std::collections::hash_map::Entry;

use ahash::AHashMap;

use crate::{
    compiler::{ast::*, lexical_analysis::TokenType},
    executable::{Executable, Instruction},
};
#[derive(Debug, Clone)]
pub struct CodeGenerator<'a> {
    num_consts: Vec<f64>,
    num_consts_map: AHashMap<&'a str, usize>,
    code: Vec<u8>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new() -> Self {
        CodeGenerator {
            code: vec![],
            num_consts: vec![],
            num_consts_map: AHashMap::new(),
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.emit_byte(instruction as u8);
    }

    fn emit_load_num_instruction(&mut self, index: u8) {
        self.emit_instruction(Instruction::LoadNumber);
        self.emit_byte(index);
    }

    fn visit(&mut self, ast: &Expr<'a>) {
        match ast {
            Expr::ExprList(el) => el.exprs.iter().for_each(|expr| self.visit(expr)),

            Expr::Program(pe) => self.visit(&pe.inner),
            Expr::Group(ge) => self.visit(&ge.inner),

            Expr::Bool(be) => self.emit_instruction(if be.value {
                Instruction::LoadTrue
            } else {
                Instruction::LoadFalse
            }),

            Expr::Number(ne) => {
                let num = ne.number;
                let lexeme = ne.token.lexeme;

                let num_index = match self.num_consts_map.entry(lexeme) {
                    Entry::Occupied(entry) => *entry.get(),

                    Entry::Vacant(entry) => {
                        self.num_consts.push(num);
                        let inserted_index = self.num_consts.len() - 1;
                        *entry.insert(inserted_index)
                    }
                };

                // todo this
                if num_index > u8::MAX as usize {
                    panic!("More number constants than u8 can hold. Add an instruction for bigger indices.");
                }

                self.emit_load_num_instruction(num_index as u8);
            }

            Expr::Print(pe) => {
                self.visit(&pe.inner);
                self.emit_instruction(Instruction::Print);
            }

            Expr::Prefix(pe) => {
                self.visit(&pe.inner);

                self.emit_instruction(match pe.operator.token_type {
                    TokenType::Minus => Instruction::Negate,
                    TokenType::Not => Instruction::Not,
                    other => panic!("this token type should not be a prefix expr: {:?}", other),
                });
            }

            Expr::Infix(ie) => {
                self.visit(&ie.left);
                self.visit(&ie.right);

                self.emit_instruction(match ie.operator.token_type {
                    TokenType::Plus => Instruction::Add,
                    TokenType::Minus => Instruction::Sub,
                    TokenType::Star => Instruction::Mul,
                    TokenType::Slash => Instruction::Div,

                    TokenType::Equal => Instruction::Equal,
                    TokenType::Less => Instruction::LessThan,
                    TokenType::LessEqual => Instruction::LessThanOrEqual,
                    TokenType::Greater => Instruction::GreaterThan,
                    TokenType::GreaterEqual => Instruction::GreaterThanOrEqual,

                    other => panic!("this token type should not be a prefix expr: {:?}", other),
                });
            }

            Expr::Var(_) => unimplemented!(),
            Expr::VarDecl(_) => unimplemented!(),
            Expr::If(_) => unimplemented!(),
        };
    }

    pub fn gen(mut self, ast: &'a Expr) -> Executable {
        self.visit(ast);

        Executable {
            code: self.code,
            num_consts: self.num_consts,
        }
    }
}
