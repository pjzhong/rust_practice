use std::{
    env, fs,
    io::{self, Write},
    process::exit,
    sync::{Arc, Mutex},
};

use rlox::{Lox, Scanner};

fn main() {
    let args = env::args().collect::<Vec<String>>();
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

fn run_file(lox: Arc<Mutex<Lox>>, file: &String) {
    match fs::read_to_string(file) {
        Ok(file) => {
            run(&file, lox);
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
    let mut scanner = Scanner::new(code, lox);
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
}
