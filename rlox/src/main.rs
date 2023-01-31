use std::{
    env, fs,
    io::{self, Write},
    process::exit,
    rc::Rc,
};

use rlox::{Interpreter, Lox, Parser, Resolver, Scanner};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let lox = Rc::new(Lox::default());

    match args.len() {
        a if a > 2 => {
            println!("Usage:rlox [script]");
            exit(64);
        }
        2 => {
            run_file(lox, &args[1]);
        }
        _ => run_prompt(lox),
    }
}

fn run_file(lox: Rc<Lox>, file: &str) {
    match fs::read_to_string(file) {
        Ok(file) => {
            run(&file, lox.clone());

            if lox.has_error() {
                exit(65);
            }

            if lox.had_runtime_error() {
                exit(70);
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            exit(65);
        }
    }
}

fn run_prompt(lox: Rc<Lox>) {
    let mut string = String::new();
    let stdio = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        if let Ok(size) = stdio.read_line(&mut string) {
            if 0 < size {
                run(&string, lox.clone());
                lox.set_error(false);
            }
        }
        string.clear();
    }
}

fn run(code: &str, lox: Rc<Lox>) {
    let scanner = Scanner::new(code, lox.clone());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens, lox.clone());

    let stmts = parser.parse();
    if lox.has_error() {
        return;
    }

    let mut interpreter = Interpreter::new(lox.clone());
    let resolver = Resolver::new(lox.clone());
    resolver.resolve(&stmts, &mut interpreter);
    if lox.has_error() {
        return;
    }
    interpreter.interpret(&stmts);
}
