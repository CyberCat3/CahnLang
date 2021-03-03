use std::{env, fs, process::exit};

use cahn_lang::{
    compiler::{
        lexical_analysis::{Lexer, TokenType},
        string_handling::StringInterner,
        CodeGenerator, Parser,
    },
    runtime::VM,
};

fn print_help() {
    eprintln!(
        "Cahn lang

USAGE:
    cahn [FLAGS] <INPUT FILE>

EXAMPLE:
    cahn ./hello_world.cahn

FLAGS:
    -s   --print-source        Prints Cahn source code to console
    -l   --print-tokens        Prints Lexer output
    -p   --print-ast           Prints the AST, the parser's output
    -c   --print-bytecode      Prints the compiled byte code
"
    );
}

#[derive(Debug, Default)]
struct Config {
    print_source: bool,
    print_tokens: bool,
    print_ast: bool,
    print_bytecode: bool,
    cahn_file: String,
}

fn get_config() -> Config {
    let mut args = env::args().peekable();

    let _exec_name = args.next().unwrap();

    if args.peek().is_none() {
        print_help();
        exit(1);
    }

    let mut config = Config::default();

    while let Some(arg) = args.next() {
        match &arg[..] {
            "-s" | "--print-source" => config.print_source = true,
            "-l" | "--print-tokens" => config.print_tokens = true,
            "-p" | "--print-ast" => config.print_ast = true,
            "-c" | "--print-bytecode" => config.print_bytecode = true,
            _ => config.cahn_file = arg,
        }
    }
    config
}

fn main() {
    let config = get_config();

    // READ SOURCE CODE
    let source_code = match fs::read_to_string(&config.cahn_file) {
        Ok(content) => content,

        Err(err) => {
            eprintln!(
                "Couldn't read '{}' due to error: {}.",
                config.cahn_file, err
            );
            exit(1);
        }
    };

    // PRINT SOURCE
    if config.print_source {
        println!("<SOURCE CODE>\n{}\n</SOURCE CODE>\n", source_code);
    }

    // CREATE INTERNER AND ARENA
    let interner = StringInterner::new();
    let arena = bumpalo::Bump::new();

    // PRINT LEXER OUTPUT
    if config.print_tokens {
        println!("<TOKENS>");
        let lexer = Lexer::new(&source_code, interner.clone());

        loop {
            let token = lexer.lex_token();
            println!("{}", token);
            if token.token_type == TokenType::Eof {
                break;
            }
        }
        println!("</TOKENS>");
    }

    // PARSE PROGRAM
    let ast = match Parser::from_str(&source_code, &arena, interner.clone()).parse_program() {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("An error occurred during parsing: {}.", err);
            exit(2);
        }
    };

    // PRINT PARSER OUTPUT
    if config.print_ast {
        println!("<AST>\n{}\n</AST>\n", ast);
    }

    // COMPILE PROGRAM
    let executable = match CodeGenerator::new(interner, config.cahn_file).gen(&ast) {
        Ok(exec) => exec,
        Err(err) => {
            eprintln!("An error occurred during compilation: {}.", err);
            exit(3);
        }
    };

    // PRINT BYTECODE
    if config.print_bytecode {
        println!("<BYTECODE>\n{}\n</BYTECODE>\n", executable);
    }

    // RUN PROGRAM
    if let Err(err) = VM::run_to_stdout(&executable) {
        eprintln!("A runtime error occurred: {}", err);
        exit(4);
    }
}
