use {
    super::*,
    crate::compiler::{lexical_analysis::Token, string_handling::StringAtom},
    bumpalo::collections::Vec,
    itertools::Itertools,
    std::fmt::{self, Debug},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Number(&'a NumberExpr),
    String(&'a StringExpr),
    Var(&'a VarExpr),
    Bool(&'a BoolExpr),
    Group(&'a GroupExpr<'a>),
    Prefix(&'a PrefixExpr<'a>),
    Infix(&'a InfixExpr<'a>),
    List(&'a ListExpr<'a>),
    Subscript(&'a SubscriptExpr<'a>),
    Call(&'a CallExpr<'a>),
    AnynFnDecl(&'a AnynFnDeclExpr<'a>),
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
            Expr::List(e) => fmt::Display::fmt(e, f),
            Expr::Subscript(e) => fmt::Display::fmt(e, f),
            Expr::Call(e) => fmt::Display::fmt(e, f),
            Expr::AnynFnDecl(e) => fmt::Display::fmt(e, f),
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

#[derive(Debug, Clone)]
pub struct ListExpr<'a> {
    pub bracket_open: Token,
    pub elements: Vec<'a, Expr<'a>>,
    pub bracket_close: Token,
}

impl<'a> ListExpr<'a> {
    pub fn new(
        bracket_open: Token,
        elements: Vec<'a, Expr<'a>>,
        bracket_close: Token,
    ) -> ListExpr<'a> {
        ListExpr {
            bracket_open,
            elements,
            bracket_close,
        }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::List(arena.alloc(self))
    }
}

impl<'a> fmt::Display for ListExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        {
            f.write_str("(list ")?;
            for elem in &self.elements {
                fmt::Display::fmt(elem, f)?;
                f.write_str(", ")?;
            }
            f.write_str(")")?;
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SubscriptExpr<'a> {
    pub subscriptee: Expr<'a>,
    pub bracket_open: Token,
    pub index: Expr<'a>,
    pub bracket_close: Token,
}

impl<'a> SubscriptExpr<'a> {
    pub fn new(
        subscriptee: Expr<'a>,
        bracket_open: Token,
        index: Expr<'a>,
        bracket_close: Token,
    ) -> SubscriptExpr<'a> {
        SubscriptExpr {
            subscriptee,
            bracket_open,
            index,
            bracket_close,
        }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Subscript(arena.alloc(self))
    }
}

impl<'a> fmt::Display for SubscriptExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("([] {} {})", self.subscriptee, self.index))
    }
}

#[derive(Debug, Clone)]
pub struct CallExpr<'a> {
    pub callee: Expr<'a>,
    pub paren_open: Token,
    pub args: Vec<'a, Expr<'a>>,
    pub paren_close: Token,
}

impl<'a> CallExpr<'a> {
    pub fn new(
        callee: Expr<'a>,
        paren_open: Token,
        args: Vec<'a, Expr<'a>>,
        paren_close: Token,
    ) -> CallExpr<'a> {
        CallExpr {
            callee,
            paren_open,
            args,
            paren_close,
        }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::Call(arena.alloc(self))
    }
}

impl<'a> fmt::Display for CallExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        {
            f.write_fmt(format_args!("(call {} ", self.callee))?;
            for arg in &self.args {
                fmt::Display::fmt(arg, f)?;
                f.write_str(", ")?;
            }
            f.write_str(")")?;
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AnynFnDeclExpr<'a> {
    pub fn_token: Token,
    pub parameters: Vec<'a, Token>,
    pub body: BlockStmt<'a>,
}

impl<'a> AnynFnDeclExpr<'a> {
    pub fn new(
        fn_token: Token,
        parameters: Vec<'a, Token>,
        body: BlockStmt<'a>,
    ) -> AnynFnDeclExpr<'a> {
        AnynFnDeclExpr {
            fn_token,
            parameters,
            body,
        }
    }

    pub fn into_expr(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
        Expr::AnynFnDecl(arena.alloc(self))
    }
}

impl<'a> fmt::Display for AnynFnDeclExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "(fn ({}) {})",
            self.parameters.iter().join(", "),
            self.body
        ))
    }
}
