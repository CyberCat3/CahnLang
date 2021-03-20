use {
    super::*,
    crate::compiler::lexical_analysis::Token,
    bumpalo::collections::Vec,
    itertools::Itertools,
    std::fmt::{self, Debug, Write},
};

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    Print(&'a PrintStmt<'a>),
    Return(&'a ReturnStmt<'a>),
    VarDecl(&'a VarDeclStmt<'a>),
    Block(&'a BlockStmt<'a>),
    StmtList(&'a StmtList<'a>),
    Program(&'a ProgramStmt<'a>),
    If(&'a IfStmt<'a>),
    While(&'a WhileStmt<'a>),
    ExprStmt(&'a ExprStmt<'a>),
    FnDecl(&'a FnDeclStmt<'a>),
}

impl<'a> fmt::Display for Stmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Print(e) => fmt::Display::fmt(e, f),
            Stmt::Return(e) => fmt::Display::fmt(e, f),
            Stmt::VarDecl(e) => fmt::Display::fmt(e, f),
            Stmt::Block(e) => fmt::Display::fmt(e, f),
            Stmt::StmtList(e) => fmt::Display::fmt(e, f),
            Stmt::Program(e) => fmt::Display::fmt(e, f),
            Stmt::If(e) => fmt::Display::fmt(e, f),
            Stmt::While(e) => fmt::Display::fmt(e, f),
            Stmt::ExprStmt(e) => fmt::Display::fmt(e, f),
            Stmt::FnDecl(e) => fmt::Display::fmt(e, f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrintStmt<'a> {
    pub print_token: Token,
    pub inner: Expr<'a>,
}

impl<'a> PrintStmt<'a> {
    pub fn new(print_token: Token, inner: Expr<'a>) -> PrintStmt<'a> {
        PrintStmt { print_token, inner }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::Print(arena.alloc(self))
    }
}

impl<'a> fmt::Display for PrintStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("(print {})", self.inner))
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStmt<'a> {
    pub return_token: Token,
    pub return_val: Option<Expr<'a>>,
}

impl<'a> ReturnStmt<'a> {
    pub fn new(return_token: Token, return_val: Option<Expr<'a>>) -> ReturnStmt<'a> {
        ReturnStmt {
            return_token,
            return_val,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::Return(arena.alloc(self))
    }
}

impl<'a> fmt::Display for ReturnStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        {
            f.write_str("(return")?;
            if let Some(return_val) = &self.return_val {
                f.write_fmt(format_args!(" {}", return_val))?;
            }
            f.write_char(')')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt<'a> {
    pub var_token: Token,
    pub identifier: Token,
    pub init_expr: Expr<'a>,
}

impl<'a> VarDeclStmt<'a> {
    pub fn new(var_token: Token, identifier: Token, init_expr: Expr<'a>) -> VarDeclStmt<'a> {
        VarDeclStmt {
            var_token,
            identifier,
            init_expr,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::VarDecl(arena.alloc(self))
    }
}

impl<'a> fmt::Display for VarDeclStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "({} {} {})",
            self.var_token.lexeme, self.identifier.lexeme, self.init_expr
        ))
    }
}

#[derive(Debug, Clone)]
pub struct BlockStmt<'a> {
    pub brace_open: Token,
    pub statements: StmtList<'a>,
    pub brace_close: Token,
}

impl<'a> BlockStmt<'a> {
    pub fn new(brace_open: Token, statements: StmtList<'a>, brace_close: Token) -> BlockStmt<'a> {
        BlockStmt {
            brace_open,
            statements,
            brace_close,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::Block(arena.alloc(self))
    }
}

impl<'a> fmt::Display for BlockStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("(block {})", self.statements))
    }
}

#[derive(Debug, Clone)]
pub struct StmtList<'a> {
    pub stmts: Vec<'a, Stmt<'a>>,
}

impl<'a> StmtList<'a> {
    pub fn new(stmts: Vec<'a, Stmt<'a>>) -> StmtList<'a> {
        StmtList { stmts }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::StmtList(arena.alloc(self))
    }
}

impl<'a> fmt::Display for StmtList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        {
            for expr in &self.stmts {
                fmt::Display::fmt(expr, f)?;
                f.write_char('\n')?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProgramStmt<'a> {
    pub statements: StmtList<'a>,
    pub eof_token: Token,
}

impl<'a> ProgramStmt<'a> {
    pub fn new(statements: StmtList<'a>, eof_token: Token) -> ProgramStmt<'a> {
        ProgramStmt {
            statements,
            eof_token,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::Program(arena.alloc(self))
    }
}

impl<'a> fmt::Display for ProgramStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("(program {})", self.statements))
    }
}

#[derive(Debug, Clone)]
pub struct IfStmt<'a> {
    pub if_token: Token,
    pub condition: Expr<'a>,
    pub then_clause: BlockStmt<'a>,
    pub else_token: Option<Token>,
    pub else_clause: Option<Stmt<'a>>,
}

impl<'a> IfStmt<'a> {
    pub fn new(
        if_token: Token,
        condition: Expr<'a>,
        then_clause: BlockStmt<'a>,
        else_token: Option<Token>,
        else_clause: Option<Stmt<'a>>,
    ) -> IfStmt<'a> {
        IfStmt {
            if_token,
            condition,
            then_clause,
            else_token,
            else_clause,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::If(arena.alloc(self))
    }
}

impl<'a> fmt::Display for IfStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "(if {} then {}",
            self.condition, self.then_clause
        ))?;
        if let Some(ec) = &self.else_clause {
            f.write_fmt(format_args!(" else {}", ec))?;
        }
        f.write_char(')')
    }
}

#[derive(Debug, Clone)]
pub struct WhileStmt<'a> {
    pub while_token: Token,
    pub condition: Expr<'a>,
    pub block: BlockStmt<'a>,
}

impl<'a> WhileStmt<'a> {
    pub fn new(while_token: Token, condition: Expr<'a>, block: BlockStmt<'a>) -> WhileStmt<'a> {
        WhileStmt {
            while_token,
            condition,
            block,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::While(arena.alloc(self))
    }
}

impl<'a> fmt::Display for WhileStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("(while {} {})", self.condition, self.block))
    }
}

#[derive(Debug, Clone)]
pub struct ExprStmt<'a> {
    pub expr: Expr<'a>,
}

impl<'a> ExprStmt<'a> {
    pub fn new(expr: Expr<'a>) -> ExprStmt<'a> {
        ExprStmt { expr }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::ExprStmt(arena.alloc(self))
    }
}

impl<'a> fmt::Display for ExprStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.expr))
    }
}

#[derive(Debug, Clone)]
pub struct FnDeclStmt<'a> {
    pub fn_token: Token,
    pub name: Token,
    pub parameters: Vec<'a, Token>,
    pub body: BlockStmt<'a>,
}

impl<'a> FnDeclStmt<'a> {
    pub fn new(
        fn_token: Token,
        name: Token,
        parameters: Vec<'a, Token>,
        body: BlockStmt<'a>,
    ) -> FnDeclStmt<'a> {
        FnDeclStmt {
            fn_token,
            name,
            parameters,
            body,
        }
    }

    pub fn into_stmt(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
        Stmt::FnDecl(arena.alloc(self))
    }
}

impl<'a> fmt::Display for FnDeclStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "(fn {} ({}) {})",
            self.name.lexeme,
            self.parameters.iter().map(|p| &p.lexeme).join(", "),
            self.body
        ))
    }
}
