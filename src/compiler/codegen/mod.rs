mod error;

use error::{CodeGenError, Result};

use std::{collections::hash_map::Entry, fmt, u32};

use ahash::AHashMap;

use crate::{
    compiler::{
        ast::*,
        lexical_analysis::{Token, TokenType},
        string_handling::{StringAtom, StringInterner},
    },
    executable::{Executable, Instruction},
};

use super::lexical_analysis::TokenPos;

#[derive(Clone)]
struct Local {
    name: Option<StringAtom>,
    scope_level: usize,
}

impl fmt::Debug for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => f.write_fmt(format_args!(
                "Local(name: {}, level: {})",
                name, self.scope_level
            )),
            None => f.write_fmt(format_args!("AnonymousLocal(level: {})", self.scope_level)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeGenerator {
    num_consts: Vec<f64>,
    num_consts_map: AHashMap<StringAtom, usize>,
    code: Vec<u8>,

    string_data: String,
    string_data_map: AHashMap<StringAtom, (u32, u32)>,

    code_map: Vec<TokenPos>,
    current_source_position: TokenPos,

    interner: StringInterner,

    locals: Vec<Local>,
    scope_level: usize,

    source_file_name: String,
}

impl CodeGenerator {
    pub fn new(interner: StringInterner, source_file_name: String) -> Self {
        CodeGenerator {
            interner,
            code: vec![],
            num_consts: vec![],
            num_consts_map: AHashMap::new(),

            current_source_position: TokenPos::new(1, 1),
            code_map: vec![],

            string_data: String::new(),
            string_data_map: AHashMap::new(),

            locals: vec![],
            scope_level: 0,
            source_file_name,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_level += 1;
    }

    fn end_scope(&mut self) {
        self.scope_level -= 1;

        while matches!(self.locals.last(), Some(local) if local.scope_level > self.scope_level) {
            self.emit_instruction(Instruction::Pop);
            self.locals.pop();
        }
    }

    fn declare_anonymous_local(&mut self) -> usize {
        let local_index = self.locals.len();
        self.locals.push(Local {
            name: None,
            scope_level: self.scope_level,
        });
        local_index
    }

    fn declare_local(&mut self, name: &StringAtom) -> usize {
        let local_index = self.locals.len();
        self.locals.push(Local {
            name: Some(name.clone()),
            scope_level: self.scope_level,
        });
        local_index
    }

    fn get_local_index_by_token(&mut self, identifier: &Token) -> Result<usize> {
        match self.get_local_index(&identifier.lexeme) {
            Some(index) => Ok(index),
            None => Err(CodeGenError::UnresolvedVariable {
                var_token: identifier.clone(),
            }),
        }
    }

    fn get_local_index(&mut self, name: &StringAtom) -> Option<usize> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .filter(|(_index, entry)| entry.name.is_some())
            .find(|(_index, entry)| entry.name.as_ref().unwrap() == name)
            .map(|(index, _entry)| index)
    }

    fn get_local(&self, index: usize) -> Option<&Local> {
        self.locals.get(index)
    }

    fn set_source_pos(&mut self, pos: TokenPos) {
        self.current_source_position = pos;
    }

    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
        self.code_map.push(self.current_source_position);
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        bytes.iter().for_each(|byte| self.emit_byte(*byte));
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.emit_byte(instruction as u8);
    }

    fn emit_load_num_lit_instruction(&mut self, num: u8) {
        self.emit_instruction(Instruction::LoadLitNum);
        self.emit_byte(num);
    }

    fn emit_get_local_instruction(&mut self, index: usize) {
        if index <= u8::MAX as usize {
            self.emit_instruction(Instruction::GetLocal);
            self.emit_byte(index as u8);
            return;
        }

        assert!(
            index <= u16::MAX as usize,
            "Too many locals! Cahn only supports up to {}, but got {}",
            u16::MAX,
            index
        );

        self.emit_instruction(Instruction::GetLocalW);
        self.emit_bytes(&(index as u16).to_le_bytes());
    }

    fn emit_set_local_instruction(&mut self, index: usize) {
        if index < u8::MAX as usize {
            self.emit_instruction(Instruction::SetLocal);
            self.emit_byte(index as u8);
            return;
        }

        assert!(
            index <= u16::MAX as usize,
            "Too many locals! Cahn only supports up to {}, but got {}",
            u16::MAX,
            index
        );

        self.emit_instruction(Instruction::SetLocalW);
        self.emit_bytes(&(index as u16).to_le_bytes());
    }

    fn emit_assignment_instructions<'a>(
        &mut self,
        target: &Expr<'a>,
        source: &Expr<'a>,
    ) -> Result<()> {
        let identifier = match target {
            Expr::Var(ve) => &ve.identifier,
            other => {
                return Err(CodeGenError::InvalidAssignmentTarget {
                    message: format!("{}", other),
                })
            }
        };

        self.visit_expr(source)?;

        self.set_source_pos(identifier.pos);
        let local = self.get_local_index_by_token(&identifier)?;
        self.emit_instruction(Instruction::Dup);
        self.emit_set_local_instruction(local);
        Ok(())
    }

    fn emit_load_number_instruction(&mut self, number: f64, lexeme: StringAtom) {
        if number >= u8::MIN as f64 && number <= u8::MAX as f64 && number.fract() == 0.0 {
            let number = number as u8;
            self.emit_load_num_lit_instruction(number);
        } else {
            let index = match self.num_consts_map.entry(lexeme) {
                Entry::Occupied(entry) => *entry.get(),

                Entry::Vacant(entry) => {
                    self.num_consts.push(number);
                    let inserted_index = self.num_consts.len() - 1;
                    *entry.insert(inserted_index)
                }
            };

            if index <= u8::MAX as usize {
                self.emit_instruction(Instruction::LoadConstNum);
                self.emit_byte(index as u8);
                return;
            }

            if index <= u16::MAX as usize {
                self.emit_instruction(Instruction::LoadConstNumW);
                self.emit_bytes(&(index as u16).to_le_bytes());
                return;
            }

            assert!(
                index <= u32::MAX as usize,
                "too many number constants! Cahn only supports op to {}, but got {}",
                u32::MAX,
                index
            );

            self.emit_instruction(Instruction::LoadConstNumWW);
            self.emit_bytes(&(index as u32).to_le_bytes());
        }
    }

    fn emit_load_string_literal_instruction(&mut self, string: &StringAtom) {
        let (start_index, end_index) = match self.string_data_map.entry(string.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),

            Entry::Vacant(entry) => {
                let start_index = self.string_data.len() as u32;

                let string_data = &mut self.string_data;
                string.run_on_str(|str| string_data.push_str(str));

                let end_index = self.string_data.len() as u32;

                let slice = (start_index, end_index);
                entry.insert(slice);
                slice
            }
        };

        self.emit_instruction(Instruction::LoadStringLiteral);
        self.emit_bytes(&start_index.to_le_bytes());
        self.emit_bytes(&end_index.to_le_bytes());
    }

    fn emit_jump_instruction(&mut self, jump_instruction: Instruction) -> usize {
        self.emit_instruction(jump_instruction);
        let patch_adress = self.code.len();
        self.emit_bytes(&0_u32.to_le_bytes());
        patch_adress
    }

    fn patch_jump_instruction(&mut self, adress: usize, jump_location: usize) {
        assert!(
            jump_location <= u32::MAX as usize,
            "jump adress ({}) is over {}",
            jump_location,
            u32::MAX,
        );

        let bytes = jump_location.to_le_bytes();
        self.code[adress] = bytes[0];
        self.code[adress + 1] = bytes[1];
        self.code[adress + 2] = bytes[2];
        self.code[adress + 3] = bytes[3];
    }

    fn visit_expr<'a>(&mut self, expr: &Expr<'a>) -> Result<()> {
        match expr {
            Expr::Group(ge) => self.visit_expr(&ge.inner)?,

            Expr::Bool(be) => {
                self.set_source_pos(be.token.pos);
                self.emit_instruction(if be.value {
                    Instruction::LoadTrue
                } else {
                    Instruction::LoadFalse
                })
            }

            Expr::Number(ne) => {
                self.set_source_pos(ne.token.pos);
                self.emit_load_number_instruction(ne.number, ne.token.lexeme.clone())
            }

            Expr::String(se) => {
                self.set_source_pos(se.token.pos);
                self.emit_load_string_literal_instruction(&se.string);
            }

            Expr::Prefix(pe) => {
                self.visit_expr(&pe.inner)?;

                self.set_source_pos(pe.operator.pos);
                self.emit_instruction(match pe.operator.token_type {
                    TokenType::Minus => Instruction::Negate,
                    TokenType::Not => Instruction::Not,
                    other => panic!("this token type should not be a prefix expr: {:?}", other),
                });
            }

            Expr::Infix(ie) => {
                if ie.operator.token_type == TokenType::ColonEqual {
                    self.emit_assignment_instructions(&ie.left, &ie.right)?;
                } else {
                    self.visit_expr(&ie.left)?;
                    self.visit_expr(&ie.right)?;

                    self.set_source_pos(ie.operator.pos);

                    self.emit_instruction(match ie.operator.token_type {
                        TokenType::Plus => Instruction::Add,
                        TokenType::Minus => Instruction::Sub,
                        TokenType::Star => Instruction::Mul,
                        TokenType::Slash => Instruction::Div,
                        TokenType::Percent => Instruction::Modulo,

                        TokenType::DoubleEqual => Instruction::Equal,
                        TokenType::Less => Instruction::LessThan,
                        TokenType::LessEqual => Instruction::LessThanOrEqual,
                        TokenType::Greater => Instruction::GreaterThan,
                        TokenType::GreaterEqual => Instruction::GreaterThanOrEqual,
                        TokenType::DoubleDot => Instruction::Concat,

                        other => panic!("this token type should not be a infix expr: {:?}", other),
                    });
                }
            }

            Expr::Var(ve) => {
                let stack_offset = self.get_local_index_by_token(&ve.identifier)?;
                self.set_source_pos(ve.identifier.pos);
                self.emit_get_local_instruction(stack_offset);
            }

            Expr::List(le) => {
                let len = le.elements.len();

                self.set_source_pos(le.bracket_open.pos);
                match len {
                    0 => self.emit_instruction(Instruction::CreateList),

                    len if len <= u8::MAX as usize => {
                        self.emit_instruction(Instruction::CreateListWithCap);
                        self.emit_byte(len as u8);
                    }

                    len if len <= u16::MAX as usize => {
                        self.emit_instruction(Instruction::CreateListWithCapW);
                        self.emit_bytes(&u16::to_le_bytes(len as u16));
                    }

                    _ => {
                        self.emit_instruction(Instruction::CreateListWithCapW);
                        self.emit_bytes(&u16::MAX.to_le_bytes());
                    }
                }

                for elem in &le.elements {
                    self.visit_expr(elem)?;
                    self.emit_instruction(Instruction::ListPush);
                }
            }

            Expr::Subscript(se) => {
                self.visit_expr(&se.subscriptee)?;
                self.visit_expr(&se.index)?;
                self.set_source_pos(se.bracket_open.pos);
                self.emit_instruction(Instruction::ListGetIndex);
            }

            Expr::Call(_) => unimplemented!("call operator () not implemented"),

            Expr::AnynFnDecl(_) => {
                unimplemented!("anynomous function declarations are really not implemented")
            }
        };

        Ok(())
    }

    fn visit_stmt_list<'a>(&mut self, stmt_list: &StmtList<'a>) -> Result<()> {
        for stmt in &stmt_list.stmts {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }

    fn visit_program_stmt<'a>(&mut self, prog_stmt: &ProgramStmt<'a>) -> Result<()> {
        self.begin_scope();
        self.visit_stmt_list(&prog_stmt.statements)?;
        self.set_source_pos(prog_stmt.eof_token.pos);
        self.end_scope();
        Ok(())
    }

    fn visit_block_stmt<'a>(&mut self, block_stmt: &BlockStmt<'a>) -> Result<()> {
        self.set_source_pos(block_stmt.brace_open.pos);
        self.begin_scope();
        self.visit_stmt_list(&block_stmt.statements)?;
        self.set_source_pos(block_stmt.brace_close.pos);
        self.end_scope();
        Ok(())
    }

    fn visit_stmt<'a>(&mut self, stmt: &Stmt<'a>) -> Result<()> {
        Ok(match stmt {
            Stmt::Program(ps) => self.visit_program_stmt(ps)?,

            Stmt::Block(bs) => self.visit_block_stmt(bs)?,

            Stmt::StmtList(sl) => self.visit_stmt_list(sl)?,

            Stmt::Print(ps) => {
                self.visit_expr(&ps.inner)?;
                self.set_source_pos(ps.print_token.pos);
                self.emit_instruction(Instruction::Print);
            }

            Stmt::VarDecl(vds) => {
                self.visit_expr(&vds.init_expr)?;
                self.set_source_pos(vds.var_token.pos);
                self.declare_local(&vds.identifier.lexeme);
            }

            Stmt::If(is) => {
                self.visit_expr(&is.condition)?;

                self.set_source_pos(is.if_token.pos);
                let then_jump = self.emit_jump_instruction(Instruction::JumpIfFalse);

                self.visit_block_stmt(&is.then_clause)?;

                let mut else_jump = None;

                if let Some(ref else_clause) = is.else_clause {
                    self.set_source_pos(else_clause.brace_open.pos);
                    else_jump = Some(self.emit_jump_instruction(Instruction::Jump));
                }

                self.patch_jump_instruction(then_jump, self.code.len());

                if let Some(else_block) = &is.else_clause {
                    self.visit_block_stmt(else_block)?;
                    self.patch_jump_instruction(else_jump.unwrap(), self.code.len());
                }
            }

            Stmt::While(ws) => {
                let start_adress = self.code.len();
                assert!(
                    start_adress <= u32::MAX as usize,
                    "while statement start is too out on the adress space."
                );
                // the adress where our while statement starts
                let start_adress = start_adress as u32;

                // compile the condition
                self.visit_expr(&ws.condition)?;

                // if the condition was false, we need to jump over the entire body, which emit the instruction for here.
                self.set_source_pos(ws.while_token.pos);
                let loop_done_adress = self.emit_jump_instruction(Instruction::JumpIfFalse);

                // compile the body
                self.visit_block_stmt(&ws.block)?;

                // when the body has executed, jump back to the start, so we actually loop.
                self.set_source_pos(ws.block.brace_close.pos);
                self.emit_instruction(Instruction::Jump);
                self.emit_bytes(&start_adress.to_le_bytes());

                // know we know were to jump to, to skip the body, so we patch the first jump.
                self.patch_jump_instruction(loop_done_adress, self.code.len());
            }

            Stmt::ExprStmt(es) => {
                self.visit_expr(&es.expr)?;
                // statements are supposed to have a stack effect of 0, so we pop
                self.emit_instruction(Instruction::Pop);
            }

            Stmt::FnDecl(_) => unimplemented!("Function declarations are so not supported yet!!!"),
        })
    }

    pub fn gen<'a>(mut self, program: &ProgramStmt<'a>) -> Result<Executable> {
        self.visit_program_stmt(program)?;

        Ok(Executable {
            string_data: self.string_data,
            source_file: self.source_file_name,
            code: self.code,
            code_map: self.code_map,
            num_consts: self.num_consts,
        })
    }
}
