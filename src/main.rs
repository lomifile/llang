use crate::error::RunError;
use compiler::types::Compiler;
use lexer::lexer::Lexer;
use parser::parser::Parser;
use types::checker::Checker;
use vm::types::Vm;

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

mod error;

fn run_line(source: &str, checker: &mut Checker, vm: &mut Vm) -> Result<(), RunError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let declarations = Parser::new(tokens).parse_program()?;
    checker.check_program(&declarations)?;

    let mut compiler = Compiler::new();
    for declaration in declarations {
        compiler.compile_declaration(declaration);
    }

    vm.interpret(compiler.chunk)?;
    Ok(())
}

fn run(source: &str) -> Result<(), RunError> {
    run_line(source, &mut Checker::new(), &mut Vm::new())
}

fn run_file(path: &str) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("could not read {path}: {err}");
            return ExitCode::from(66);
        }
    };

    match run(&source) {
        Ok(()) => {
            println!("> Done");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(70)
        }
    }
}

fn repl() -> ExitCode {
    let mut vm = Vm::new();
    let mut checker = Checker::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut line = String::new();

    println!("> JSL compiler <");
    loop {
        print!("> ");
        if io::stdout().flush().is_err() {
            return ExitCode::from(74);
        }

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::from(74);
            }
        }

        if line.trim().is_empty() {
            break;
        }

        if let Err(err) = run_line(&line, &mut checker, &mut vm) {
            eprintln!("{err}");
        }
    }

    println!();
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);

    match (args.next(), args.next()) {
        (None, _) => repl(),
        (Some(path), None) => run_file(&path),
        (Some(_), Some(_)) => {
            eprintln!("usage: jsl [file]");
            ExitCode::from(64)
        }
    }
}
