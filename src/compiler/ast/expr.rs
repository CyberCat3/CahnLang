use std::fmt::{self, Debug};

use crate::compiler::{lexical_analysis::Token, string_handling::StringAtom};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Number(&'a NumberExpr),
    String(&'a StringExpr),
    Var(&'a VarExpr),
    Bool(&'a BoolExpr),
    Group(&'a GroupExpr<'a>),
    Prefix(&'a PrefixExpr<'a>),
    Infix(&'a InfixExpr<'a>),
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(e) => fmt::Display::fmt(e, f),
            Expr::String(e) => fmt::Display::fmt(e, f),
            Expr::Var(e) => fmt::Display::fmt(e, f),
            Expr::Bool(e) => fmt::Display::fmt(e, f),
            Expr::Group(e) => fmt::Display::fmt(e, f),
            Expr::Prefix(e) => fmt::Display::fmt(e, f),
            Expr::Infix(e) => fmt::Display::fmt(e, f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumberExpr {
    pub token: Token,
    pub number: f64,
}

impl NumberExpr {
    pub fn new(token: Token, number: f64) -> NumberExpr {
        NumberExpr { token, number }
    }

    pub fn into_expr<'a>(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Number(arena.alloc(self))
    }
}

impl fmt::Display for NumberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.token.lexeme))
    }
}

#[derive(Debug, Clone)]
pub struct StringExpr {
    pub token: Token,
    pub string: StringAtom,
}

impl StringExpr {
    pub fn new(token: Token, string: StringAtom) -> StringExpr {
        StringExpr { token, string }
    }

    pub fn into_expr<'a>(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::String(arena.alloc(self))
    }
}

impl fmt::Display for StringExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.token.lexeme))
    }
}

#[derive(Debug, Clone)]
pub struct VarExpr {
    pub identifier: Token,
}

impl VarExpr {
    pub fn new(identifier: Token) -> VarExpr {
        VarExpr { identifier }
    }

    pub fn into_expr<'a>(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Var(arena.alloc(self))
    }
}

impl fmt::Display for VarExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.identifier.lexeme))
    }
}

#[derive(Debug, Clone)]
pub struct BoolExpr {
    pub token: Token,
    pub value: bool,
}

impl BoolExpr {
    pub fn new(token: Token, value: bool) -> BoolExpr {
        BoolExpr { token, value }
    }

    pub fn into_expr<'a>(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Bool(arena.alloc(self))
    }
}

impl fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.token.lexeme))
    }
}

#[derive(Debug, Clone)]
pub struct GroupExpr<'a> {
    pub paren_open: Token,
    pub inner: Expr<'a>,
    pub paren_close: Token,
}

impl<'a> GroupExpr<'a> {
    pub fn new(paren_open: Token, inner: Expr<'a>, paren_close: Token) -> GroupExpr<'a> {
        GroupExpr {
            paren_open,
            inner,
            paren_close,
        }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Group(arena.alloc(self))
    }
}

impl<'a> fmt::Display for GroupExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({})", self.inner))
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpr<'a> {
    pub operator: Token,
    pub inner: Expr<'a>,
}

impl<'a> PrefixExpr<'a> {
    pub fn new(operator: Token, inner: Expr<'a>) -> PrefixExpr<'a> {
        PrefixExpr { operator, inner }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Prefix(arena.alloc(self))
    }
}

impl<'a> fmt::Display for PrefixExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({} {})", self.operator.lexeme, self.inner))
    }
}

#[derive(Debug, Clone)]
pub struct InfixExpr<'a> {
    pub left: Expr<'a>,
    pub operator: Token,
    pub right: Expr<'a>,
}

impl<'a> InfixExpr<'a> {
    pub fn new(left: Expr<'a>, operator: Token, right: Expr<'a>) -> InfixExpr<'a> {
        InfixExpr {
            left,
            operator,
            right,
        }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Infix(arena.alloc(self))
    }
}

impl<'a> fmt::Display for InfixExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "({} {} {})",
            self.operator.lexeme, self.left, self.right
        ))
    }
}
