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

    fn finish_block_stmt(&self, brace_open: Token) -> Result<BlockStmt<'a>> {
        let content = self.parse_statement_list()?;
        let brace_close = self.expect(TokenType::BraceClose, || {
            "expected '}' to close block".into()
        })?;
        Ok(BlockStmt::new(brace_open, content, brace_close))
    }

    fn finish_var_decl_statement(&self, var_token: Token) -> Result<VarDeclStmt<'a>> {
        let ident = self.expect(TokenType::Identifier, || {
            "expected identifier after variable declaration".into()
        })?;

        let _assignment_operator = self.expect(TokenType::ColonEqual, || {
            "expected := after variable name".into()
        })?;

        let expr = self.parse_expression()?;

        Ok(VarDeclStmt::new(var_token, ident, expr))
    }

    fn finish_if_stmt(&self, if_token: Token) -> Result<IfStmt<'a>> {
        let condition = self.parse_expression()?;

        let brace_open = self.expect(TokenType::BraceOpen, || {
            "expected '{' after if-condition".into()
        })?;

        let then_block = self.finish_block_stmt(brace_open)?;

        let (else_block, else_token) = if let Some(else_token) = self.check_advance(TokenType::Else)
        {
            if let Some(else_if_token) = self.check_advance(TokenType::If) {
                let else_if_block = self.finish_if_stmt(else_if_token)?;
                (Some(else_if_block.into_stmt(self.arena)), Some(else_token))
            } else {
                let brace_open =
                    self.expect(TokenType::BraceOpen, || "expected '{' after else".into())?;

                let else_block = self.finish_block_stmt(brace_open)?;
                (Some(else_block.into_stmt(self.arena)), Some(else_token))
            }
        } else {
            (None, None)
        };

        Ok(IfStmt::new(
            if_token, condition, then_block, else_token, else_block,
        ))
    }

    fn finish_while_stmt(&self, while_token: Token) -> Result<WhileStmt<'a>> {
        let condition = self.parse_expression()?;

        let brace_open = self.expect(TokenType::BraceOpen, || {
            "expected '{' after condition in while statement".into()
        })?;

        let while_body = self.finish_block_stmt(brace_open)?;

        Ok(WhileStmt::new(while_token, condition, while_body))
    }

    fn finish_fn_decl_stmt(&self, fn_token: Token) -> Result<FnDeclStmt<'a>> {
        let identifier = self.expect(TokenType::Identifier, || {
            "expected function name after 'fn' in statement".into()
        })?;

        let mut parameters = bumpalo::vec![in self.arena];

        let _paren_open = self.expect(TokenType::ParenOpen, || {
            "expected '(' after function name".into()
        })?;

        loop {
            if self.check_ttype(TokenType::ParenClose) {
                break;
            }

            parameters
                .push(self.expect(TokenType::Identifier, || "expected paramater name".into())?);

            if self.check_advance(TokenType::Comma).is_none() {
                break;
            }
        }

        let _paren_close = self.expect(TokenType::ParenClose, || {
            "expected ')' after parameter list".into()
        })?;

        let brace_open = self.expect(TokenType::BraceOpen, || "expected function body".into())?;
        let fn_body = self.finish_block_stmt(brace_open)?;

        Ok(FnDeclStmt::new(fn_token, identifier, parameters, fn_body))
    }

    fn finish_anyn_fn_decl_expr(&self, fn_token: Token) -> Result<AnynFnDeclExpr<'a>> {
        unimplemented!("{}", fn_token)
    }

    fn parse_statement(&self) -> Result<Stmt<'a>> {
        let node = match self.peek_token().token_type {
            TokenType::Let => self
                .finish_var_decl_statement(self.advance_token())?
                .into_stmt(self.arena),

            TokenType::Print => self
                .finish_print_statement(self.advance_token())?
                .into_stmt(self.arena),

            TokenType::BraceOpen => self
                .finish_block_stmt(self.advance_token())?
                .into_stmt(self.arena),

            TokenType::If => self
                .finish_if_stmt(self.advance_token())?
                .into_stmt(self.arena),

            TokenType::While => self
                .finish_while_stmt(self.advance_token())?
                .into_stmt(self.arena),

            TokenType::Fn => self
                .finish_fn_decl_stmt(self.advance_token())?
                .into_stmt(self.arena),

            TokenType::Return => self
                .finish_return_statement(self.advance_token())?
                .into_stmt(self.arena),

            _ => ExprStmt::new(self.parse_expression()?).into_stmt(self.arena),
        };

        // eat optional semicolons
        while self.check_advance(TokenType::Semicolon).is_some() {}

        Ok(node)
    }

    fn finish_print_statement(&self, print_token: Token) -> Result<PrintStmt<'a>> {
        let expr = self.parse_expression()?;
        Ok(PrintStmt::new(print_token, expr))
    }

    fn finish_return_statement(&self, return_token: Token) -> Result<ReturnStmt<'a>> {
        let expr = if self.check_ttype_any(token_groups::BLOCK_ENDINGS) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        Ok(ReturnStmt::new(return_token, expr))
    }

    fn finish_group_expression(&self, paren_open: Token) -> Result<GroupExpr<'a>> {
        let expr = self.parse_expression()?;
        let paren_close = self.expect(TokenType::ParenClose, || {
            String::from("expected a closing parenthesis")
        })?;
        return Ok(GroupExpr::new(paren_open, expr, paren_close));
    }

    fn finish_list_expression(&self, bracket_open: Token) -> Result<ListExpr<'a>> {
        // zero-element list
        if let Some(bracket_close) = self.check_advance(TokenType::BracketClose) {
            return Ok(ListExpr::new(
                bracket_open,
                bumpalo::vec![in self.arena],
                bracket_close,
            ));
        }

        // single-element list
        let first_elem = self.parse_expression()?;

        if let Some(bracket_close) = self.check_advance(TokenType::BracketClose) {
            return Ok(ListExpr::new(
                bracket_open,
                bumpalo::vec![in self.arena; first_elem],
                bracket_close,
            ));
        }

        // multi-element list
        let mut elements = bumpalo::vec![in self.arena; first_elem];

        while self.check_advance(TokenType::Comma).is_some() {
            if let Some(bracket_close) = self.check_advance(TokenType::BracketClose) {
                return Ok(ListExpr::new(bracket_open, elements, bracket_close));
            }
            elements.push(self.parse_expression()?);
        }

        let bracket_close = self.expect(TokenType::BracketClose, || {
            "expected ']' to terminate list".into()
        })?;

        Ok(ListExpr::new(bracket_open, elements, bracket_close))
    }

    fn parse_expression(&self) -> Result<Expr<'a>> {
        self.parse_assignment()
    }

    fn parse_assignment(&self) -> Result<Expr<'a>> {
        let expr = self.parse_and()?;

        if let Some(assignment_operator) = self.check_advance(TokenType::ColonEqual) {
            let right_expr = self.parse_and()?;

            if let Some(chained_operator) = self.check_advance(TokenType::ColonEqual) {
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

        while let Some(operator) = self.check_advance_any(&[
            TokenType::Star,
            TokenType::Slash,
            TokenType::DoubleSlash,
            TokenType::Percent,
        ]) {
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
        let expr = self.parse_call()?;

        if let Some(operator) = self.check_advance(TokenType::DoubleStar) {
            Ok(InfixExpr::new(expr, operator, self.parse_unary()?).into_expr(self.arena))
        } else {
            Ok(expr)
        }
    }

    fn parse_call(&self) -> Result<Expr<'a>> {
        let mut expr = self.parse_atom()?;

        'outer: while let Some(open) =
            self.check_advance_any(&[TokenType::ParenOpen, TokenType::BracketOpen])
        {
            match open.token_type {
                TokenType::BracketOpen => {
                    let bracket_open = open;
                    let index = self.parse_expression()?;

                    let bracket_close = self.expect(TokenType::BracketClose, || {
                        "expected ] to close subscript operator".into()
                    })?;

                    expr = SubscriptExpr::new(expr, bracket_open, index, bracket_close)
                        .into_expr(self.arena);
                }

                TokenType::ParenOpen => {
                    let paren_open = open;
                    // zero arg
                    if let Some(paren_close) = self.check_advance(TokenType::ParenClose) {
                        expr = CallExpr::new(
                            expr,
                            paren_open,
                            bumpalo::vec![in self.arena],
                            paren_close,
                        )
                        .into_expr(self.arena);
                        continue 'outer;
                    }

                    let mut args = bumpalo::vec![in self.arena; self.parse_expression()?];

                    // one arg
                    if let Some(paren_close) = self.check_advance(TokenType::ParenClose) {
                        expr = CallExpr::new(expr, paren_open, args, paren_close)
                            .into_expr(self.arena);

                        continue 'outer;
                    }

                    // multi arg
                    while self.check_advance(TokenType::Comma).is_some() {
                        if let Some(paren_close) = self.check_advance(TokenType::ParenClose) {
                            expr = CallExpr::new(expr, paren_open, args, paren_close)
                                .into_expr(self.arena);

                            continue 'outer;
                        }
                        args.push(self.parse_expression()?);
                    }

                    let paren_close = self.expect(TokenType::ParenClose, || {
                        "expected ')' to close argument list".into()
                    })?;

                    expr = CallExpr::new(expr, paren_open, args, paren_close).into_expr(self.arena)
                }
                _ => unreachable!(),
            }
        }
        Ok(expr)
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

            TokenType::Fn => self.finish_anyn_fn_decl_expr(token)?.into_expr(self.arena),

            TokenType::ParenOpen => self.finish_group_expression(token)?.into_expr(self.arena),

            TokenType::BracketOpen => self.finish_list_expression(token)?.into_expr(self.arena),
            _ => {
                return Err(ParseError::BadToken {
                    message: "expected either a literal, a variable or (".into(),
                    token,
                })
            }
        })
    }
}
