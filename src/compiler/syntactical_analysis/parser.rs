use std::cell::Cell;
use crate::compiler::{
    lexical_analysis::{Lexer, Token, TokenType, token_groups},
    syntactical_analysis::error::{ParseError, Result},
    ast::*,
};

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek_token: Cell<Token<'a>>,
    arena: &'a bumpalo::Bump,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, arena: &'a bumpalo::Bump) -> Self {
        let t = lexer.lex_token();
        Parser {
            lexer,
            arena,
            peek_token: Cell::new(t),
        }
    }

    pub fn from_str(source: &'a str, arena: &'a bumpalo::Bump) -> Self {
        let lexer = Lexer::new(source);
        Self::new(lexer, arena)
    }

    fn peek_token(&self) -> Token<'a> {
        self.peek_token.get()
    }

    fn advance_token(&self) -> Token<'a> {
        let peek_token = self.peek_token();
        self.peek_token.set(self.lexer.lex_token());
        peek_token
    }

    fn check_ttype(&self, expected: TokenType) -> bool {
        self.peek_token().token_type == expected
    }

    fn check_ttype_any(&self, expected: &[TokenType]) -> bool {
        expected.iter().any(|etype| self.check_ttype(*etype))
    }

    fn check_advance(&self, expected: TokenType) -> Option<Token<'a>> {
        if self.check_ttype(expected) {
            Some(self.advance_token())
        } else {
            None
        }
    }

    fn check_advance_any(&self, expected: &[TokenType]) -> Option<Token<'a>> {
        for etype in expected {
            match self.check_advance(*etype) {
                Some(t) => return Some(t),
                None => {}
            }
        }
        None
    }

    fn expect<T: FnOnce() -> String>(
        &self,
        expected: TokenType,
        message_func: T,
    ) -> Result<Token<'a>> {
        if self.check_ttype(expected) {
            Ok(self.advance_token())
        } else {
            Err(ParseError::BadToken {message: message_func(), token: self.advance_token()})
        }
    }

    fn expect_any<T: FnOnce() -> String>(
        &self,
        expected: &[TokenType],
        message_func: T,
    ) -> Result<Token<'a>> {
        for etype in expected {
            if self.check_ttype(*etype) {
                return Ok(self.advance_token());
            }
        }
        Err(ParseError::BadToken {message: message_func(), token: self.advance_token()})
    }

    pub fn parse_program(&self) -> Result<Expr<'a>> {
        let exprs = self.parse_expression_list()?;
        let eof = self.expect(TokenType::Eof, || "The program should end here".into())?;
        Ok(ProgramExpr::new(self.arena, exprs, eof))
    }

    fn parse_expression_list(&self) -> Result<Expr<'a>> {
        let expr = self.parse_expression()?;

        if self.check_ttype_any(token_groups::BLOCK_ENDINGS) {
            return Ok(expr);
        }

        let mut exprs = bumpalo::vec![in self.arena; expr];

        while !self.check_ttype_any(token_groups::BLOCK_ENDINGS) {
            exprs.push(self.parse_expression()?);
        }

        Ok(ExprList::new(self.arena, exprs))
    }

    fn finish_var_decl_expresion(&self, var_token: Token<'a>) -> Result<Expr<'a>> {
        let ident = self.expect(TokenType::Identifier, || {
            "expected identifier after variable declaration".into()
        })?;

        let expr = self.parse_expression()?;

        Ok(VarDeclExpr::new(self.arena, var_token, ident, expr))
    }

    fn finish_if_expression(&self, if_token: Token<'a>) -> Result<Expr<'a>> {
        let condition = self.parse_expression()?;
        let _then_token = self.expect(TokenType::Then, || {
            "expected 'then' after if-condition".into()
        })?;
        let then_clause = self.parse_expression_list()?;

        if let Some(end_token) = self.check_advance(TokenType::End) {
            return Ok(IfExpr::new(
                self.arena,
                if_token,
                condition,
                then_clause,
                None,
                end_token,
            ));
        };

        if let Some(_else_token) = self.check_advance(TokenType::Else) {
            let else_clause = self.parse_expression_list()?;
            let end_token =
                self.expect(TokenType::End, || "expected 'end' after else-clause".into())?;
            return Ok(IfExpr::new(
                self.arena,
                if_token,
                condition,
                then_clause,
                Some(else_clause),
                end_token,
            ));
        };

        if let Some(else_if_token) = self.check_advance(TokenType::ElseIf) {
            let else_if_expr = self.finish_if_expression(else_if_token)?;
            return Ok(IfExpr::new(
                self.arena,
                if_token,
                condition,
                then_clause,
                Some(else_if_expr),
                else_if_token,
            ));
        }

        Err(ParseError::BadToken {
            token: self.advance_token(),
            message: "expected either 'else-if', 'else' or 'end' after then-clause".into(),
        })
    }

    fn finish_print_expression(&self, print_token: Token<'a>) -> Result<Expr<'a>> {
        let expr = self.parse_expression()?;
        Ok(PrintExpr::new(self.arena, print_token, expr))
    }

    fn finish_group_expression(&self, paren_open: Token<'a>) -> Result<Expr<'a>> {
        let expr = self.parse_expression()?;
        let paren_close = self.expect(TokenType::ParenClose, || {
            String::from("expected a closing parenthesis")
        })?;
        return Ok(GroupExpr::new(self.arena, paren_open, expr, paren_close));
    }

    fn parse_expression(&self) -> Result<Expr<'a>> {
        self.parse_assignment()
    }

    fn parse_assignment(&self) -> Result<Expr<'a>> {
        let expr = self.parse_and()?;
        
        if let Some(assignment_operator) = self.check_advance(TokenType::SemicolonEqual) {
            let right_expr = self.parse_and()?;
 
            if let Some(chained_operator) = self.check_advance(TokenType::SemicolonEqual) {
                return Err(ParseError::ChainingAssignmentOperator { operator: chained_operator });
            }

            return Ok(InfixExpr::new(self.arena, expr, assignment_operator, right_expr));
        }
        Ok(expr)
    }

    fn parse_and(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_or()?;

        while let Some(operator) = self.check_advance(TokenType::And) {
            expr = InfixExpr::new(self.arena, expr, operator, self.parse_or()?);
        }

        Ok(expr)
    }

    fn parse_or(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_comparison()?;

        while let Some(operator) = self.check_advance(TokenType::Or) {
            expr = InfixExpr::new(self.arena, expr, operator, self.parse_comparison()?);
        }

        Ok(expr)
    }

    fn parse_comparison(&self) -> Result<Expr<'a>> {
        let expr = self.parse_addition()?;

        if let Some(operator) = self.check_advance_any(token_groups::COMPARISON_OPERATORS) {
            let right_expr = self.parse_addition()?;
 
            if let Some(chained_operator) = self.check_advance_any(token_groups::COMPARISON_OPERATORS) {
                return Err(ParseError::ChainingComparisonOperator { operator: chained_operator });
            }

            return Ok(InfixExpr::new(self.arena, expr, operator, right_expr));
        }
        Ok(expr)
    }

    fn parse_addition(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_multiplication()?;

        while let Some(operator) = self.check_advance_any(&[TokenType::Plus, TokenType::Minus]) {
            expr = InfixExpr::new(self.arena, expr, operator, self.parse_multiplication()?);
        }

        Ok(expr)
    }

    fn parse_multiplication(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_unary()?;

        while let Some(operator) =
            self.check_advance_any(&[TokenType::Star, TokenType::Slash, TokenType::DoubleSlash])
        {
            expr = InfixExpr::new(self.arena, expr, operator, self.parse_unary()?);
        }

        Ok(expr)
    }

    fn parse_unary(&self) -> Result<Expr<'a>> {
        if let Some(operator) = self.check_advance_any(token_groups::PREFIX_OPERATORS) {
            Ok(PrefixExpr::new(self.arena, operator, self.parse_unary()?))
        } else {
            self.parse_exponent()
        }
    }

    fn parse_exponent(&self) -> Result<Expr<'a>> {
        let expr = self.parse_atom()?;

        if let Some(operator) = self.check_advance(TokenType::DoubleStar) {
            Ok(InfixExpr::new(
                self.arena,
                expr,
                operator,
                self.parse_unary()?,
            ))
        } else {
            Ok(expr)
        }
    }

    fn parse_atom(&self) -> Result<Expr<'a>> {
        if let Some(token) = self.check_advance_any(token_groups::LITERALS) {
            return Ok(match token.token_type {
                TokenType::Number => {
                    NumberExpr::new(self.arena, token, token.lexeme.parse::<f64>().unwrap())
                }
                TokenType::True => BoolExpr::new(self.arena, token, true),
                TokenType::False => BoolExpr::new(self.arena, token, false),
                _ => unreachable!(),
            });
        }

        match self.peek_token().token_type {
            TokenType::Identifier => Ok(VarExpr::new(self.arena, self.advance_token())),

            TokenType::ParenOpen => self.finish_group_expression(self.advance_token()),

            TokenType::Let => self.finish_var_decl_expresion(self.advance_token()),

            TokenType::If => self.finish_if_expression(self.advance_token()),

            TokenType::Print => self.finish_print_expression(self.advance_token()),

            _ => Err(ParseError::BadToken {token: self.peek_token(), message: "expected some expression here".into()}),
        }
    }
}
