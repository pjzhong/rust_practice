use std::io::{self, Read};

fn main() {
    for b in io::stdin().bytes() {
        if let Ok(b) = b {
            let c = b as char;
            println!("{}", c);
        }
    }
}
