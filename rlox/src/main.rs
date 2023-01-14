use std::{
    env, fs,
    io::{self, Write},
    process::exit,
    sync::{Arc, Mutex},
};

use rlox::{Interpreter, Lox, Parser, Scanner};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let args = vec!["aaa", "I:\\work\\rust_practice\\rlox\\test\\scopes.lox"];
    let lox = Arc::new(Mutex::new(Lox::default()));

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

fn run_file(lox: Arc<Mutex<Lox>>, file: &str) {
    match fs::read_to_string(file) {
        Ok(file) => {
            run(&file, lox.clone());

            if let Ok(lox) = lox.lock() {
                if lox.has_error {
                    exit(65);
                }

                if lox.had_runtime_error {
                    exit(70);
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            exit(65);
        }
    }
}

fn run_prompt(lox: Arc<Mutex<Lox>>) {
    let mut string = String::new();
    let stdio = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        if let Ok(size) = stdio.read_line(&mut string) {
            if 0 < size {
                run(&string, lox.clone());
                if let Ok(mut lox) = lox.lock() {
                    lox.has_error = false;
                }
            }
        }
        string.clear();
    }
}

fn run(code: &str, lox: Arc<Mutex<Lox>>) {
    let scanner = Scanner::new(code, lox.clone());
    let tokens = scanner.scan_tokens();
    println!("{:?}", tokens);
    let mut parser = Parser::new(tokens, lox.clone());

    let stmts = parser.parse();

    // let printer = AstPrinter;
    // println!("{:?}", expr);
    // println!("{:?}", printer.visit_expr(&expr));

    let mut interpreter = Interpreter::new(lox);
    interpreter.interpret(&stmts);
}
