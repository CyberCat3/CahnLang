mod error;

use error::{CodeGenError, Result};

use std::{collections::hash_map::Entry, fmt, mem, u32};

use ahash::AHashMap;

use crate::{
    compiler::{
        ast::*,
        lexical_analysis::{Token, TokenType},
        string_handling::StringAtom,
    },
    executable::{Executable, Instruction},
};

#[derive(Clone)]
struct Local {
    identifier: Token,
    scope_level: usize,
}

impl fmt::Debug for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Local(name: {}, level: {})",
            self.identifier.lexeme, self.scope_level
        ))
    }
}

#[derive(Debug, Clone)]
pub struct CodeGenerator {
    num_consts: Vec<f64>,
    num_consts_map: AHashMap<StringAtom, usize>,
    code: Vec<u8>,

    locals: Vec<Local>,
    scope_level: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            code: vec![],
            num_consts: vec![],
            num_consts_map: AHashMap::new(),

            locals: vec![],
            scope_level: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_level += 1;
        println!("began scope, all locals: {:?}", self.locals);
    }

    fn end_scope(&mut self) {
        self.scope_level -= 1;

        let mut locals_evicted = 0;
        while matches!(self.locals.last(), Some(local) if local.scope_level > self.scope_level) {
            self.emit_instruction(Instruction::Pop);
            self.locals.pop();
            locals_evicted += 1;
        }
        println!(
            "ended scope, evicted {} locals. all locals: {:?}",
            locals_evicted, self.locals
        );
    }

    fn declare_local(&mut self, identifier: Token) {
        self.locals.push(Local {
            identifier: identifier.clone(),
            scope_level: self.scope_level,
        });
        println!(
            "declared local {}. all locals: {:?}",
            identifier.lexeme, self.locals
        );
    }

    fn get_local(&mut self, identifier: Token) -> Result<usize> {
        let local = self
            .locals
            .iter()
            .enumerate()
            .rev()
            .find(|(_index, entry)| entry.identifier.lexeme == identifier.lexeme);

        match local {
            Some((index, _local)) => Ok(index),
            None => Err(CodeGenError::UnresolvedVariable {
                identifier: identifier.lexeme,
                index: identifier.index,
            }),
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.emit_byte(instruction as u8);
    }

    fn emit_load_num_lit_instruction(&mut self, num: u8) {
        self.emit_instruction(Instruction::LoadLitNum);
        self.emit_byte(num);
    }

    fn emit_load_num_instruction(&mut self, index: usize) {
        if index < u8::MAX as usize {
            self.emit_instruction(Instruction::LoadConstNum);
            self.emit_byte(index as u8);
            return;
        }

        if index < u16::MAX as usize {
            self.emit_instruction(Instruction::LoadConstNumW);
            self.emit_bytes(&(index as u16).to_le_bytes());
            return;
        }

        if index < u32::MAX as usize {
            self.emit_instruction(Instruction::LoadConstNumWW);
            self.emit_bytes(&(index as u32).to_le_bytes());
            return;
        }

        panic!(
            "So many number constants! Cahn only supports up to {}, but got {}",
            u32::MAX,
            index
        );
    }

    fn emit_get_local_instruction(&mut self, index: usize) {
        if index < u8::MAX as usize {
            self.emit_instruction(Instruction::GetLocal);
            self.emit_byte(index as u8);
            return;
        }

        if index < u16::MAX as usize {
            self.emit_instruction(Instruction::GetLocalW);
            self.emit_bytes(&(index as u16).to_le_bytes());
            return;
        }

        panic!(
            "So many locals! Cahn only supports up to {}, but got {}",
            u16::MAX,
            index
        );
    }

    fn visit<'a>(&mut self, ast: &Expr<'a>) -> Result<()> {
        // println!("visit called on: {}", ast);
        match ast {
            Expr::Block(be) => {
                self.begin_scope();
                self.visit(&be.inner_expr)?;
                self.end_scope();
                self.emit_instruction(Instruction::LoadNil);
            }

            Expr::ExprList(el) => {
                for expr in el.exprs.iter() {
                    self.visit(expr)?;
                    self.emit_instruction(Instruction::Pop);
                }
            }

            Expr::Program(pe) => {
                self.begin_scope();
                self.visit(&pe.inner)?;
                self.end_scope();
            }

            Expr::Group(ge) => self.visit(&ge.inner)?,

            Expr::Bool(be) => self.emit_instruction(if be.value {
                Instruction::LoadTrue
            } else {
                Instruction::LoadFalse
            }),

            Expr::Number(ne) => {
                let num = ne.number;
                let lexeme = ne.token.lexeme.clone();

                if num >= u8::MIN as f64 && num <= u8::MAX as f64 && num.fract() == 0.0 {
                    let num = num as u8;
                    self.emit_load_num_lit_instruction(num);
                } else {
                    let num_index = match self.num_consts_map.entry(lexeme) {
                        Entry::Occupied(entry) => *entry.get(),

                        Entry::Vacant(entry) => {
                            self.num_consts.push(num);
                            let inserted_index = self.num_consts.len() - 1;
                            *entry.insert(inserted_index)
                        }
                    };
                    self.emit_load_num_instruction(num_index);
                }
            }

            Expr::Print(pe) => {
                self.visit(&pe.inner)?;
                self.emit_instruction(Instruction::Print);
                self.emit_instruction(Instruction::LoadNil);
            }

            Expr::Prefix(pe) => {
                self.visit(&pe.inner)?;

                self.emit_instruction(match pe.operator.token_type {
                    TokenType::Minus => Instruction::Negate,
                    TokenType::Not => Instruction::Not,
                    other => panic!("this token type should not be a prefix expr: {:?}", other),
                });
            }

            Expr::Infix(ie) => {
                self.visit(&ie.left)?;
                self.visit(&ie.right)?;

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

            Expr::VarDecl(vde) => {
                self.visit(&vde.init_expr)?;
                self.declare_local(vde.identifier.clone());
                self.emit_instruction(Instruction::LoadNil);
            }

            Expr::Var(ve) => {
                let stack_offset = self.get_local(ve.identifier.clone())?;
                self.emit_get_local_instruction(stack_offset);
            }

            Expr::If(_) => unimplemented!(),
        };

        Ok(())
    }

    fn optimize(code: Vec<u8>) -> Vec<u8> {
        let mut code_opt = vec![];

        let mut iter = code.into_iter().peekable();

        while let Some(cop) = iter.next() {
            let instruction: Instruction = unsafe { mem::transmute(cop) };

            match instruction {
                Instruction::LoadConstNum
                | Instruction::LoadLitNum
                | Instruction::GetLocal
                | Instruction::SetLocal => {
                    code_opt.push(cop);
                    code_opt.push(iter.next().unwrap());
                }

                Instruction::LoadConstNumW | Instruction::GetLocalW | Instruction::SetLocalW => {
                    code_opt.push(cop);
                    code_opt.push(iter.next().unwrap());
                    code_opt.push(iter.next().unwrap());
                }

                Instruction::LoadConstNumWW => {
                    code_opt.push(cop);
                    code_opt.push(iter.next().unwrap());
                    code_opt.push(iter.next().unwrap());
                    code_opt.push(iter.next().unwrap());
                    code_opt.push(iter.next().unwrap());
                }

                Instruction::LoadNil | Instruction::Dup => loop {
                    if let Some(next) = iter.peek() {
                        let next_instruction: Instruction = unsafe { mem::transmute(*next) };
                        if next_instruction == Instruction::Pop {
                            iter.next();
                            break;
                        }
                    }
                    code_opt.push(cop);
                    break;
                },

                _ => {
                    code_opt.push(cop);
                }
            }
        }

        code_opt
    }

    pub fn gen<'a>(mut self, ast: &'a Expr) -> Result<Executable> {
        self.visit(ast)?;

        Ok(Executable {
            code: Self::optimize(self.code),
            // code: self.code,
            num_consts: self.num_consts,
        })
    }
}
