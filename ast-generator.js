const exprs = [
    {
        name: "NumberExpr",
        ename: "Number",
        format: "{}", fargs: "e.token.lexeme",
        fields: {
            token: "Token",
            number: "f64"
        }
    },
    {
        name: "VarExpr",
        ename: "Var",
        format: "{}", fargs: "e.identifier.lexeme",
        fields: {
            identifier: "Token",
        }
    },
    {
        name: "BoolExpr",
        ename: "Bool",
        format: "{}", fargs: "e.token.lexeme",
        fields: {
            token: "Token",
            value: "bool"
        }
    },
    {
        name: "GroupExpr",
        ename: "Group",
        format: "({})", fargs: "e.inner",
        fields: {
            paren_open: "Token",
            inner: "Expr",
            paren_close: "Token",
        }
    },
    {
        name: "PrefixExpr",
        ename: "Prefix",
        format: "({} {})", fargs: "e.operator.lexeme, e.inner",
        fields: {
            operator: "Token",
            inner: "Expr"
        }
    },
    {
        name: "InfixExpr",
        ename: "Infix",
        format: "({} {} {})", fargs: "e.operator.lexeme, e.left, e.right",
        fields: {
            left: "Expr",
            operator: "Token",
            right: "Expr",
        }
    },
    {
        name: "PrintExpr",
        ename: "Print",
        format: "(print {})", fargs: "e.inner",
        fields: {
            print_token: "Token",
            inner: "Expr",
        }
    },
    {
        name: "VarDeclExpr",
        ename: "VarDecl",
        format: "({} {} {})", fargs: "e.var_token.lexeme, e.identifier.lexeme, e.init_expr",
        fields: {
            var_token: "Token",
            identifier: "Token",
            init_expr: "Expr",
        }
    },
    {
        name: "BlockExpr",
        ename: "Block",
        format: "(block {})", fargs: "e.inner_expr",
        fields: {
            open_token: "Token",
            inner_expr: "Expr",
            close_token: "Token",
        }
    },
    {
        name: "ExprList",
        ename: "ExprList",
        format_custom: `{
                for expr in &e.exprs {
                    fmt::Display::fmt(expr, f)?;
                    f.write_char('\\n')?;       
                };
            },\n`,
        fields: {
            exprs: "Vec<'a, Expr<'a>>"
        },
    },
    {
        name: "ProgramExpr",
        ename: "Program",
        format: "(program {})", fargs: "e.inner",
        fields: {
            inner: "Expr",
            eof_token: "Token"
        }
    },
    {
        name: "IfExpr",
        ename: "If",
        format_custom: `{
                f.write_fmt(format_args!("(if {} then {}", e.condition, e.then_clause))?;
                if let Some(ec) = &e.else_clause {
                    f.write_fmt(format_args!(" else {}", ec))?;
                }
                f.write_char(')')?;
            },\n`,
        fields: {
            if_token: "Token",
            condition: "Expr",
            then_clause: "Expr",
            else_clause: "Option<Expr<'a>>",
            end_token: "Token",
        }
    }
];

const output = [];

output.push(
`use std::fmt::{self, Debug, Write};

use super::lexical_analysis::Token;

use bumpalo::collections::Vec;

#[derive(Debug, Clone)]
pub enum Expr<'a> {\n`);
for (const expr of exprs) {
    output.push(`    ${expr.ename}(&'a ${expr.name}<'a>),\n`)
}

output.push(
`}

impl<'a> fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
`);

for (let i = 0; i < exprs.length; ++i) {
    const expr = exprs[i];
    output.push(`            Expr::${expr.ename}(e) => `);
    if (expr.format_custom) {
        output.push(expr.format_custom);
    } else {
        output.push(`f.write_fmt(format_args!("${expr.format}", ${expr.fargs}))?,\n`)
    }
    if (i + 1 < exprs.length) output.push('\n');
}

output.push(
`        }
    Ok(())
    }
}
`);

for (const expr of exprs) {
    output.push(
`#[derive(Debug, Clone)]
pub struct ${expr.name}<'a> {\n`);
        
    for (const key of Object.keys(expr.fields)) {
        const val = expr.fields[key];
        output.push(`    pub ${key}: ${val}`);
        if (val.endsWith("Token") || val.endsWith("Expr")) output.push("<'a>");
        output.push(",\n");
    }

    output.push("}\n");

    output.push(`impl<'a> ${expr.name}<'a> {\n    pub fn new(arena: &'a bumpalo::Bump, `);

    const keys = Object.keys(expr.fields);
    for (let i = 0; i < keys.length; ++i) {
        const key = keys[i];
        const val = expr.fields[key];
        output.push(`${key}: ${val}`);
        if (val.endsWith("Token") || val.endsWith("Expr")) output.push("<'a>");
        if (i + 1 < keys.length) output.push(", ");
    }

    // output.push(`pub fn new_${expr.nname}<'a>(arena: &'a bumpalo::Bump, `);
    
    output.push(") -> Expr<'a> {\n");

    output.push(`        Expr::${expr.ename}(arena.alloc_with(|| ${expr.name} { `);
    
    for (let i = 0; i < keys.length; ++i) {
        const key = keys[i];
        output.push(key);
        if (i + 1 < keys.length) output.push(", ");
    }

    output.push(" }))\n");
    output.push("    }\n");
    output.push("}\n\n");
}

const fs = require("fs");

fs.writeFileSync("./src/compiler/ast.rs", output.join(""));