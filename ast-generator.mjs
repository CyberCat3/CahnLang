console.log("generating ast...");

const exprs = [
    {
        name: "NumberExpr",
        ename: "Number",
        format: "{}", fargs: "self.token.lexeme",
        fields: {
            token: "Token",
            number: "f64"
        }
    },
    {
        name: "StringExpr",
        ename: "String",
        format: "{}", fargs: "self.token.lexeme",
        fields: {
            token: "Token",
            string: "StringAtom"
        }
    },
    {
        name: "VarExpr",
        ename: "Var",
        format: "{}", fargs: "self.identifier.lexeme",
        fields: {
            identifier: "Token",
        }
    },
    {
        name: "BoolExpr",
        ename: "Bool",
        format: "{}", fargs: "self.token.lexeme",
        fields: {
            token: "Token",
            value: "bool"
        }
    },
    {
        name: "GroupExpr",
        ename: "Group",
        format: "({})", fargs: "self.inner",
        fields: {
            paren_open: "Token",
            inner: "Expr<'a>",
            paren_close: "Token",
        }
    },
    {
        name: "PrefixExpr",
        ename: "Prefix",
        format: "({} {})", fargs: "self.operator.lexeme, self.inner",
        fields: {
            operator: "Token",
            inner: "Expr<'a>"
        }
    },
    {
        name: "InfixExpr",
        ename: "Infix",
        format: "({} {} {})", fargs: "self.operator.lexeme, self.left, self.right",
        fields: {
            left: "Expr<'a>",
            operator: "Token",
            right: "Expr<'a>",
        }
    },
    {
        name: "ListExpr",
        ename: "List",
        format_custom: `{
            f.write_str("(list ")?;
            for elem in &self.elements {
                fmt::Display::fmt(elem, f)?;
                f.write_str(", ")?;      
            };
            f.write_str(")")?;
        }; Ok(())\n`,
        fields: {
            bracket_open: "Token",
            elements: "Vec<'a, Expr<'a>>",
            bracket_close: "Token",
        }
    },
    {
        name: "SubscriptExpr",
        ename: "Subscript",
        format: "([] {} {})", fargs: "self.subscriptee, self.index",
        fields: {
            subscriptee: "Expr<'a>",
            bracket_open: "Token",
            index: "Expr<'a>",
            bracket_close: "Token",
        }
    },
    {
        name: "CallExpr",
        ename: "Call",
        format_custom: `{
            f.write_fmt(format_args!("(call {} ", self.callee))?;
            for arg in &self.args {
                fmt::Display::fmt(arg, f)?;
                f.write_str(", ")?;      
            };
            f.write_str(")")?;
        }; Ok(())\n`,
        fields: {
            callee: "Expr<'a>",
            paren_open: "Token",
            args: "Vec<'a, Expr<'a>>",
            paren_close: "Token"
        },
    },
    {
        name: "AnynFnDeclExpr",
        ename: "AnynFnDecl",
        format: "(fn ({}) {})", fargs: `self.parameters.iter().join(", "), self.body`,
        fields: {
            fn_token: "Token",
            parameters: "Vec<'a, Token>",
            body: "BlockStmt<'a>",
        }
    }
];

const stmts = [
    {
        name: "PrintStmt",
        ename: "Print",
        format: "(print {})", fargs: "self.inner",
        fields: {
            print_token: "Token",
            inner: "Expr<'a>",
        }
    },
    {
        name: "VarDeclStmt",
        ename: "VarDecl",
        format: "({} {} {})", fargs: "self.var_token.lexeme, self.identifier.lexeme, self.init_expr",
        fields: {
            var_token: "Token",
            identifier: "Token",
            init_expr: "Expr<'a>",
        }
    },
    {
        name: "BlockStmt",
        ename: "Block",
        format: "(block {})", fargs: "self.statements",
        fields: {
            brace_open: "Token",
            statements: "StmtList<'a>",
            brace_close: "Token",
        }
    },
    {
        name: "StmtList",
        ename: "StmtList",
        format_custom: `{
                for expr in &self.stmts {
                    fmt::Display::fmt(expr, f)?;
                    f.write_char('\\n')?;       
                };
            }; Ok(())\n`,
        fields: {
            stmts: "Vec<'a, Stmt<'a>>"
        },
    },
    {
        name: "ProgramStmt",
        ename: "Program",
        format: "(program {})", fargs: "self.statements",
        fields: {
            statements: "StmtList<'a>",
            eof_token: "Token"
        }
    },
    {
        name: "IfStmt",
        ename: "If",
        format_custom: `
            f.write_fmt(format_args!("(if {} then {}", self.condition, self.then_clause))?;
            if let Some(ec) = &self.else_clause {
                f.write_fmt(format_args!(" else {}", ec))?;
            }
            f.write_char(')')\n`,
        fields: {
            if_token: "Token",
            condition: "Expr<'a>",
            then_clause: "BlockStmt<'a>",
            else_clause: "Option<BlockStmt<'a>>",
        }
    },
    {
        name: "WhileStmt",
        ename: "While",
        format: "(while {} {})", fargs: "self.condition, self.block",
        fields: {
            while_token: "Token",
            condition: "Expr<'a>",
            block: "BlockStmt<'a>",
        }
    },
    {
        name: "ExprStmt",
        ename: "ExprStmt",
        format: "{}", fargs: "self.expr",
        fields: {
            expr: "Expr<'a>"
        }
    },
    {
        name: "FnDeclStmt",
        ename: "FnDecl",
        format: "(fn {} ({}) {})", fargs: `self.name, self.parameters.iter().join(", "), self.body`,
        fields: {
            fn_token: "Token",
            name: "Token",
            parameters: "Vec<'a, Token>",
            body: "BlockStmt<'a>",
        }
    }
]

function structContainsLifeTime(struct) {
    let structLifetime = false;
    for (const val of Object.values(struct.fields)) {
        if (val.includes("<'a>")) {
            structLifetime = true;
        }
    }
    return structLifetime;
}

function createExprStructAndImpl(expr) {
    const structLifetime = structContainsLifeTime(expr);

    const structAttachedLifetime = structLifetime ? "<'a>" : "";

    const structName = `${expr.name}${structAttachedLifetime}`

    const fields = Object.entries(expr.fields).map(([name, type]) => `    pub ${name}: ${type},`).join("\n");

    const parameters = Object.entries(expr.fields).map(([name, type]) => `${name}: ${type}`).join(", ");

    const parameterNames = Object.keys(expr.fields).join(", ");

    const formatCode = expr.format_custom ??
    `f.write_fmt(format_args!("${expr.format}", ${expr.fargs}))`

    const string = `
    #[derive(Debug, Clone)]
        pub struct ${structName} {
        ${fields}
        }

        impl${structAttachedLifetime} ${structName} {
            pub fn new(${parameters}) -> ${structName} {
                ${expr.name} { ${parameterNames} }
            }

            pub fn into_expr${structLifetime ? "" : "<'a>"}(self, arena: &'a bumpalo::Bump) -> Expr<'a> {
                Expr::${expr.ename}(arena.alloc(self))
            }
        }

        impl${structAttachedLifetime} fmt::Display for ${structName} {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                ${formatCode}
            }
        }`;

    return string;
}

function createExprs(exprs) {
    const fileString = `
        use {
            super::*,
            crate::compiler::{lexical_analysis::Token, string_handling::StringAtom},
            bumpalo::collections::Vec,
            itertools::Itertools,
            std::fmt::{self, Debug},
        };
    
        #[derive(Debug, Clone)]
        pub enum Expr<'a> {
            ${exprs.map(expr => `${expr.ename}(&'a ${expr.name}${structContainsLifeTime(expr) ? "<'a>" : ""}),`).join("\n")}
        }

        impl<'a> fmt::Display for Expr<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    ${exprs.map(expr => `Expr::${expr.ename}(e) => fmt::Display::fmt(e, f),`).join("\n")}
                }
            }
        }

        ${exprs.map(expr => createExprStructAndImpl(expr)).join("\n")}`;

    return fileString;
}

function createStmtStructAndImpl(stmt) {
    const structLifetime = structContainsLifeTime(stmt);

    const structAttachedLifetime = structLifetime ? "<'a>" : "";

    const structName = `${stmt.name}${structAttachedLifetime}`

    const fields = Object.entries(stmt.fields).map(([name, type]) => `    pub ${name}: ${type},`).join("\n");

    const parameters = Object.entries(stmt.fields).map(([name, type]) => `${name}: ${type}`).join(", ");

    const parameterNames = Object.keys(stmt.fields).join(", ");

    const formatCode = stmt.format_custom ??
    `f.write_fmt(format_args!("${stmt.format}", ${stmt.fargs}))`

    const string = `
    #[derive(Debug, Clone)]
        pub struct ${structName} {
        ${fields}
        }

        impl${structAttachedLifetime} ${structName} {
            pub fn new(${parameters}) -> ${structName} {
                ${stmt.name} { ${parameterNames} }
            }

            pub fn into_stmt${structLifetime ? "" : "<'a>"}(self, arena: &'a bumpalo::Bump) -> Stmt<'a> {
                Stmt::${stmt.ename}(arena.alloc(self))
            }
        }

        impl${structAttachedLifetime} fmt::Display for ${structName} {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                ${formatCode}
            }
        }`;

    return string;
}

function createStmts(stmts) {
    const fileString = `
        use {
            super::*,
            crate::compiler::lexical_analysis::Token,
            bumpalo::collections::Vec,
            itertools::Itertools,
            std::fmt::{self, Debug, Write},
        };    
    
        #[derive(Debug, Clone)]
        pub enum Stmt<'a> {
            ${stmts.map(stmt => `${stmt.ename}(&'a ${stmt.name}${structContainsLifeTime(stmt) ? "<'a>" : ""}),`).join("\n")}
        }

        impl<'a> fmt::Display for Stmt<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    ${stmts.map(stmt => `Stmt::${stmt.ename}(e) => fmt::Display::fmt(e, f),`).join("\n")}
                }
            }
        }

        ${stmts.map(stmt => createStmtStructAndImpl(stmt)).join("\n")}`;

    return fileString;
}

import { execSync } from "child_process";
import { writeFileSync } from "fs";
writeFileSync("./src/compiler/ast/expr.rs", createExprs(exprs));
console.log("Created expressions");
writeFileSync("./src/compiler/ast/stmt.rs", createStmts(stmts));
console.log("Created statements");

execSync("rustfmt ./src/compiler/ast/expr.rs");
console.log("Formatted expr.rs");

execSync("rustfmt ./src/compiler/ast/stmt.rs");
console.log("Formatted stmt.rs");