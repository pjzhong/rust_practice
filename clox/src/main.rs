use std::{
    env, fs,
    io::{self, Write},
    process::{self, exit},
};

use clox::{interpret, InterpretResult, Vm};

fn main() {
    let mut vm = Vm::new();

    let args = env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        repl(&mut vm)
    } else if args.len() == 2 {
        run_file(&mut vm, &args[1])
    } else {
        eprintln!(" Usage: clox [path]");
        process::exit(64);
    }
}

fn repl(vm: &mut Vm) {
    let mut string = String::new();
    let stdio = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        if let Ok(size) = stdio.read_line(&mut string) {
            if 0 < size {
                interpret(&string);
            }
        }
        string.clear();
    }
}

fn run_file(_vm: &mut Vm, file: &str) {
    match fs::read_to_string(file) {
        Ok(file) => {
            //let result = InterpretResult::Ok;
            let result = interpret(&file);

            match result {
                InterpretResult::Ok => (),
                InterpretResult::CompileError => exit(65),
                InterpretResult::RuntimeError => exit(70),
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            exit(65);
        }
    }
}
