use std::fs::File;
use std::io::Read;
use std::path::Path;

fn file_double<P: AsRef<Path>>(file_path: P) -> i32 {
    let mut file = File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let n: i32 = contents.trim().parse().unwrap();
    2 * n
}

fn main() {
    let double = file_double("foobar");
    println!("{}", double);
}
