use string_handling::StringInterner;

use crate::compiler::{
    ast::*,
    lexical_analysis::{token_groups, Lexer, Token, TokenType},
    string_handling,
    syntactical_analysis::error::{ParseError, Result},
};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek_token: RefCell<Token>,
    arena: &'a bumpalo::Bump,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, arena: &'a bumpalo::Bump) -> Self {
        let t = lexer.lex_token();
        Parser {
            lexer,
            arena,
            peek_token: RefCell::new(t),
        }
    }

    pub fn from_str(source: &'a str, arena: &'a bumpalo::Bump, interner: StringInterner) -> Self {
        let lexer = Lexer::new(source, interner);
        Self::new(lexer, arena)
    }

    fn peek_token(&self) -> Token {
        self.peek_token.borrow().clone()
    }

    fn advance_token(&self) -> Token {
        let peek_token = self.peek_token();
        *self.peek_token.borrow_mut() = self.lexer.lex_token();
        peek_token
    }

    fn check_ttype(&self, expected: TokenType) -> bool {
        self.peek_token().token_type == expected
    }

    fn check_ttype_any(&self, expected: &[TokenType]) -> bool {
        expected.iter().any(|etype| self.check_ttype(*etype))
    }

    fn check_advance(&self, expected: TokenType) -> Option<Token> {
        if self.check_ttype(expected) {
            Some(self.advance_token())
        } else {
            None
        }
    }

    fn check_advance_any(&self, expected: &[TokenType]) -> Option<Token> {
        for etype in expected {
            match self.check_advance(*etype) {
                Some(t) => return Some(t),
                None => {}
            }
        }
        None
    }

    fn expect<T: FnOnce() -> String>(&self, expected: TokenType, message_func: T) -> Result<Token> {
        if self.check_ttype(expected) {
            Ok(self.advance_token())
        } else {
            Err(ParseError::BadToken {
                message: message_func(),
                token: self.advance_token(),
            })
        }
    }

    // fn expect_any<T: FnOnce() -> String>(
    //     &self,
    //     expected: &[TokenType],
    //     message_func: T,
    // ) -> Result<Token> {
    //     for etype in expected {
    //         if self.check_ttype(*etype) {
    //             return Ok(self.advance_token());
    //         }
    //     }
    //     Err(ParseError::BadToken {
    //         message: message_func(),
    //         token: self.advance_token(),
    //     })
    // }

    pub fn parse_program(&self) -> Result<ProgramStmt<'a>> {
        let exprs = self.parse_statement_list()?;
        let eof = self.expect(TokenType::Eof, || "The program should end here".into())?;
        Ok(ProgramStmt::new(exprs, eof))
    }

    fn parse_statement_list(&self) -> Result<StmtList<'a>> {
        let mut stmts = bumpalo::vec![in self.arena; self.parse_statement()?];

        while !self.check_ttype_any(token_groups::BLOCK_ENDINGS) {
            stmts.push(self.parse_statement()?);
        }

        Ok(StmtList::new(stmts))
    }

    fn finish_block_stmt(&self, block_token: Token) -> Result<BlockStmt<'a>> {
        let content = self.parse_statement_list()?;
        let end_token = self.expect(TokenType::End, || {
            "expected 'end' to close explicit block".into()
        })?;
        Ok(BlockStmt::new(block_token, content, end_token))
    }

    fn finish_var_decl_statement(&self, var_token: Token) -> Result<VarDeclStmt<'a>> {
        let ident = self.expect(TokenType::Identifier, || {
            "expected identifier after variable declaration".into()
        })?;

        let _assignment_operator = self.expect(TokenType::SemicolonEqual, || {
            "expected := after variable name".into()
        })?;

        let expr = self.parse_expression()?;

        Ok(VarDeclStmt::new(var_token, ident, expr))
    }

    fn finish_if_stmt(&self, if_token: Token) -> Result<IfStmt<'a>> {
        let condition = self.parse_expression()?;
        let then_token = self.expect(TokenType::Then, || {
            "expected 'then' after if-condition".into()
        })?;

        let then_stmts = self.parse_statement_list()?;

        if let Some(else_token) = self.check_advance(TokenType::Else) {
            let then_block = BlockStmt::new(if_token.clone(), then_stmts, else_token.clone());
            let else_stmts = self.parse_statement_list()?;

            let end_token =
                self.expect(TokenType::End, || "expected end after else-clause".into())?;

            let else_block = BlockStmt::new(else_token.clone(), else_stmts, end_token.clone());

            Ok(IfStmt::new(
                if_token,
                condition,
                then_block,
                Some(else_token),
                Some(else_block),
                end_token,
            ))
        } else {
            let end_token =
                self.expect(TokenType::End, || "expected end after then-clause".into())?;
            let then_block = BlockStmt::new(then_token, then_stmts, end_token.clone());

            Ok(IfStmt::new(
                if_token, condition, then_block, None, None, end_token,
            ))
        }
    }

    fn parse_statement(&self) -> Result<Stmt<'a>> {
        let token = self.advance_token();

        Ok(match token.token_type {
            TokenType::Let => self.finish_var_decl_statement(token)?.into_stmt(self.arena),

            TokenType::Print => self.finish_print_statement(token)?.into_stmt(self.arena),

            TokenType::Block => self.finish_block_stmt(token)?.into_stmt(self.arena),

            TokenType::If => self.finish_if_stmt(token)?.into_stmt(self.arena),

            _ => {
                return Err(ParseError::UnexpectedToken {
                    token,
                    message: "expected statement".into(),
                })
            }
        })
    }

    // fn finish_if_expression(&self, if_token: Token) -> Result<IfExpr<'a>> {
    //     let condition = self.parse_expression()?;
    //     let then_token = self.expect(TokenType::Then, || {
    //         "expected 'then' after if-condition".into()
    //     })?;

    //     let then_exprs = self.parse_expression_list()?;

    //     if let Some(end_token) = self.check_advance(TokenType::End) {
    //         return Ok(IfExpr::new(
    //             if_token,
    //             condition,
    //             BlockExpr::new(then_token, then_exprs, end_token.clone()),
    //             None,
    //             end_token,
    //         ));
    //     };

    //     if let Some(else_token) = self.check_advance(TokenType::Else) {
    //         let else_exprs = self.parse_expression_list()?;

    //         let end_token =
    //             self.expect(TokenType::End, || "expected 'end' after else-clause".into())?;

    //         return Ok(IfExpr::new(
    //             if_token,
    //             condition,
    //             BlockExpr::new(then_token, then_exprs, else_token.clone()),
    //             Some(BlockExpr::new(else_token, else_exprs, end_token.clone())),
    //             end_token,
    //         ));
    //     };

    //     if let Some(else_if_token) = self.check_advance(TokenType::ElseIf) {
    //         let else_if_expr = self.finish_if_expression(else_if_token.clone())?;
    //         return Ok(IfExpr::new(
    //             if_token.clone(),
    //             condition,
    //             BlockExpr::new(else_if_token.clone(), then_exprs, else_if_expr.if_token),
    //             Some(BlockExpr::new(
    //                 if_token,
    //                 ExprList::new(bumpalo::vec![in self.arena; self.parse_expression()?]),
    //                 else_if_token.clone(),
    //             )),
    //             else_if_token,
    //         ));
    //     }

    //     Err(ParseError::BadToken {
    //         token: self.advance_token(),
    //         message: "expected either 'else-if', 'else' or 'end' after then-clause".into(),
    //     })
    // }

    fn finish_print_statement(&self, print_token: Token) -> Result<PrintStmt<'a>> {
        let expr = self.parse_expression()?;
        Ok(PrintStmt::new(print_token, expr))
    }

    fn finish_group_expression(&self, paren_open: Token) -> Result<GroupExpr<'a>> {
        let expr = self.parse_expression()?;
        let paren_close = self.expect(TokenType::ParenClose, || {
            String::from("expected a closing parenthesis")
        })?;
        return Ok(GroupExpr::new(paren_open, expr, paren_close));
    }

    fn parse_expression(&self) -> Result<Expr<'a>> {
        self.parse_assignment()
    }

    fn parse_assignment(&self) -> Result<Expr<'a>> {
        let expr = self.parse_and()?;

        if let Some(assignment_operator) = self.check_advance(TokenType::SemicolonEqual) {
            let right_expr = self.parse_and()?;

            if let Some(chained_operator) = self.check_advance(TokenType::SemicolonEqual) {
                return Err(ParseError::ChainingAssignmentOperator {
                    operator: chained_operator,
                });
            }

            return Ok(InfixExpr::new(expr, assignment_operator, right_expr).into_expr(self.arena));
        }
        Ok(expr)
    }

    fn parse_and(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_or()?;

        while let Some(operator) = self.check_advance(TokenType::And) {
            expr = InfixExpr::new(expr, operator, self.parse_or()?).into_expr(self.arena);
        }

        Ok(expr)
    }

    fn parse_or(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_comparison()?;

        while let Some(operator) = self.check_advance(TokenType::Or) {
            expr = InfixExpr::new(expr, operator, self.parse_comparison()?).into_expr(self.arena);
        }

        Ok(expr)
    }

    fn parse_comparison(&self) -> Result<Expr<'a>> {
        let expr = self.parse_concatenation()?;

        if let Some(operator) = self.check_advance_any(token_groups::COMPARISON_OPERATORS) {
            let right_expr = self.parse_concatenation()?;

            if let Some(chained_operator) =
                self.check_advance_any(token_groups::COMPARISON_OPERATORS)
            {
                return Err(ParseError::ChainingComparisonOperator {
                    operator: chained_operator,
                });
            }

            return Ok(InfixExpr::new(expr, operator, right_expr).into_expr(self.arena));
        }
        Ok(expr)
    }

    fn parse_concatenation(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_addition()?;

        while let Some(operator) = self.check_advance(TokenType::DoubleDot) {
            expr = InfixExpr::new(expr, operator, self.parse_addition()?).into_expr(self.arena);
        }

        Ok(expr)
    }

    fn parse_addition(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_multiplication()?;

        while let Some(operator) = self.check_advance_any(&[TokenType::Plus, TokenType::Minus]) {
            expr =
                InfixExpr::new(expr, operator, self.parse_multiplication()?).into_expr(self.arena);
        }

        Ok(expr)
    }

    fn parse_multiplication(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_unary()?;

        while let Some(operator) =
            self.check_advance_any(&[TokenType::Star, TokenType::Slash, TokenType::DoubleSlash])
        {
            expr = InfixExpr::new(expr, operator, self.parse_unary()?).into_expr(self.arena);
        }

        Ok(expr)
    }

    fn parse_unary(&self) -> Result<Expr<'a>> {
        if let Some(operator) = self.check_advance_any(token_groups::PREFIX_OPERATORS) {
            Ok(PrefixExpr::new(operator, self.parse_unary()?).into_expr(self.arena))
        } else {
            self.parse_exponent()
        }
    }

    fn parse_exponent(&self) -> Result<Expr<'a>> {
        let expr = self.parse_atom()?;

        if let Some(operator) = self.check_advance(TokenType::DoubleStar) {
            Ok(InfixExpr::new(expr, operator, self.parse_unary()?).into_expr(self.arena))
        } else {
            Ok(expr)
        }
    }

    fn parse_atom(&self) -> Result<Expr<'a>> {
        let token = self.advance_token();

        Ok(match token.token_type {
            TokenType::Number => NumberExpr::new(
                token.clone(),
                token
                    .lexeme
                    .run_on_str(|str| str.parse())
                    .expect("Lexer shouldn't tokenize invalid numbers"),
            )
            .into_expr(self.arena),

            TokenType::String => {
                // cut is for removing ""
                StringExpr::new(token.clone(), token.lexeme.cut(1, 1)).into_expr(self.arena)
            }

            TokenType::True => BoolExpr::new(token, true).into_expr(self.arena),
            TokenType::False => BoolExpr::new(token, false).into_expr(self.arena),
            TokenType::Identifier => VarExpr::new(token).into_expr(self.arena),

            TokenType::ParenOpen => self.finish_group_expression(token)?.into_expr(self.arena),

            _ => {
                return Err(ParseError::BadToken {
                    message: "expected either a literal, a variable or (".into(),
                    token,
                })
            }
        })
    }
}
