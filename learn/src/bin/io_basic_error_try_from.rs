use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn file_double<P: AsRef<Path>>(file_path: P) -> Result<i32, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let n = contents.trim().parse::<i32>()?;

    Ok(2 * n)
}

fn main() {
    match file_double("D:\\work\\rust_practice\\foobar") {
        Ok(n) => println!("{}", n),
        Err(err) => println!("{}", err),
    }
}
