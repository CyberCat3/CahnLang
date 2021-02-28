use std::fmt::{self, Debug, Write};

use super::lexical_analysis::Token;

use bumpalo::collections::Vec;

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Number(&'a NumberExpr),
    Var(&'a VarExpr),
    Bool(&'a BoolExpr),
    Group(&'a GroupExpr<'a>),
    Prefix(&'a PrefixExpr<'a>),
    Infix(&'a InfixExpr<'a>),
    Print(&'a PrintExpr<'a>),
    VarDecl(&'a VarDeclExpr<'a>),
    Block(&'a BlockExpr<'a>),
    ExprList(&'a ExprList<'a>),
    Program(&'a ProgramExpr<'a>),
    If(&'a IfExpr<'a>),
}

impl<'a> fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(e) => f.write_fmt(format_args!("{}", e.token.lexeme))?,

            Expr::Var(e) => f.write_fmt(format_args!("{}", e.identifier.lexeme))?,

            Expr::Bool(e) => f.write_fmt(format_args!("{}", e.token.lexeme))?,

            Expr::Group(e) => f.write_fmt(format_args!("({})", e.inner))?,

            Expr::Prefix(e) => f.write_fmt(format_args!("({} {})", e.operator.lexeme, e.inner))?,

            Expr::Infix(e) => f.write_fmt(format_args!(
                "({} {} {})",
                e.operator.lexeme, e.left, e.right
            ))?,

            Expr::Print(e) => f.write_fmt(format_args!("(print {})", e.inner))?,

            Expr::VarDecl(e) => f.write_fmt(format_args!(
                "({} {} {})",
                e.var_token.lexeme, e.identifier.lexeme, e.init_expr
            ))?,

            Expr::Block(e) => f.write_fmt(format_args!("(block {})", e.inner_expr))?,

            Expr::ExprList(e) => {
                for expr in &e.exprs {
                    fmt::Display::fmt(expr, f)?;
                    f.write_char('\n')?;
                }
            }

            Expr::Program(e) => f.write_fmt(format_args!("(program {})", e.inner))?,

            Expr::If(e) => {
                f.write_fmt(format_args!("(if {} then {}", e.condition, e.then_clause))?;
                if let Some(ec) = &e.else_clause {
                    f.write_fmt(format_args!(" else {}", ec))?;
                }
                f.write_char(')')?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct NumberExpr {
    pub token: Token,
    pub number: f64,
}
impl NumberExpr {
    pub fn new<'a>(arena: &'a bumpalo::Bump, token: Token, number: f64) -> Expr<'a> {
        Expr::Number(arena.alloc_with(|| NumberExpr { token, number }))
    }
}

#[derive(Debug, Clone)]
pub struct VarExpr {
    pub identifier: Token,
}
impl VarExpr {
    pub fn new<'a>(arena: &'a bumpalo::Bump, identifier: Token) -> Expr<'a> {
        Expr::Var(arena.alloc_with(|| VarExpr { identifier }))
    }
}

#[derive(Debug, Clone)]
pub struct BoolExpr {
    pub token: Token,
    pub value: bool,
}
impl BoolExpr {
    pub fn new<'a>(arena: &'a bumpalo::Bump, token: Token, value: bool) -> Expr<'a> {
        Expr::Bool(arena.alloc_with(|| BoolExpr { token, value }))
    }
}

#[derive(Debug, Clone)]
pub struct GroupExpr<'a> {
    pub paren_open: Token,
    pub inner: Expr<'a>,
    pub paren_close: Token,
}
impl<'a> GroupExpr<'a> {
    pub fn new(
        arena: &'a bumpalo::Bump,
        paren_open: Token,
        inner: Expr<'a>,
        paren_close: Token,
    ) -> Expr<'a> {
        Expr::Group(arena.alloc_with(|| GroupExpr {
            paren_open,
            inner,
            paren_close,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpr<'a> {
    pub operator: Token,
    pub inner: Expr<'a>,
}
impl<'a> PrefixExpr<'a> {
    pub fn new(arena: &'a bumpalo::Bump, operator: Token, inner: Expr<'a>) -> Expr<'a> {
        Expr::Prefix(arena.alloc_with(|| PrefixExpr { operator, inner }))
    }
}

#[derive(Debug, Clone)]
pub struct InfixExpr<'a> {
    pub left: Expr<'a>,
    pub operator: Token,
    pub right: Expr<'a>,
}
impl<'a> InfixExpr<'a> {
    pub fn new(
        arena: &'a bumpalo::Bump,
        left: Expr<'a>,
        operator: Token,
        right: Expr<'a>,
    ) -> Expr<'a> {
        Expr::Infix(arena.alloc_with(|| InfixExpr {
            left,
            operator,
            right,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct PrintExpr<'a> {
    pub print_token: Token,
    pub inner: Expr<'a>,
}
impl<'a> PrintExpr<'a> {
    pub fn new(arena: &'a bumpalo::Bump, print_token: Token, inner: Expr<'a>) -> Expr<'a> {
        Expr::Print(arena.alloc_with(|| PrintExpr { print_token, inner }))
    }
}

#[derive(Debug, Clone)]
pub struct VarDeclExpr<'a> {
    pub var_token: Token,
    pub identifier: Token,
    pub init_expr: Expr<'a>,
}
impl<'a> VarDeclExpr<'a> {
    pub fn new(
        arena: &'a bumpalo::Bump,
        var_token: Token,
        identifier: Token,
        init_expr: Expr<'a>,
    ) -> Expr<'a> {
        Expr::VarDecl(arena.alloc_with(|| VarDeclExpr {
            var_token,
            identifier,
            init_expr,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpr<'a> {
    pub open_token: Token,
    pub inner_expr: Expr<'a>,
    pub close_token: Token,
}
impl<'a> BlockExpr<'a> {
    pub fn new(
        arena: &'a bumpalo::Bump,
        open_token: Token,
        inner_expr: Expr<'a>,
        close_token: Token,
    ) -> Expr<'a> {
        Expr::Block(arena.alloc_with(|| BlockExpr {
            open_token,
            inner_expr,
            close_token,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct ExprList<'a> {
    pub exprs: Vec<'a, Expr<'a>>,
}
impl<'a> ExprList<'a> {
    pub fn new(arena: &'a bumpalo::Bump, exprs: Vec<'a, Expr<'a>>) -> Expr<'a> {
        Expr::ExprList(arena.alloc_with(|| ExprList { exprs }))
    }
}

#[derive(Debug, Clone)]
pub struct ProgramExpr<'a> {
    pub inner: Expr<'a>,
    pub eof_token: Token,
}
impl<'a> ProgramExpr<'a> {
    pub fn new(arena: &'a bumpalo::Bump, inner: Expr<'a>, eof_token: Token) -> Expr<'a> {
        Expr::Program(arena.alloc_with(|| ProgramExpr { inner, eof_token }))
    }
}

#[derive(Debug, Clone)]
pub struct IfExpr<'a> {
    pub if_token: Token,
    pub condition: Expr<'a>,
    pub then_clause: Expr<'a>,
    pub else_clause: Option<Expr<'a>>,
    pub end_token: Token,
}
impl<'a> IfExpr<'a> {
    pub fn new(
        arena: &'a bumpalo::Bump,
        if_token: Token,
        condition: Expr<'a>,
        then_clause: Expr<'a>,
        else_clause: Option<Expr<'a>>,
        end_token: Token,
    ) -> Expr<'a> {
        Expr::If(arena.alloc_with(|| IfExpr {
            if_token,
            condition,
            then_clause,
            else_clause,
            end_token,
        }))
    }
}
