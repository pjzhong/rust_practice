use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let mut buf = [0; 1024];
    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all("*1\r\n$4\r\nping\r\n$4\r\nhello fasdfasdfasdfasdfasdf\r\n".as_bytes())
            {
                println!("{} write error", e);
                return;
            }
            if let Ok(size) = stream.read(&mut buf) {
                let str = String::from_utf8(buf[0..size].to_vec()).unwrap();
                println!("{}", str);
            }
        }
        Err(e) => {
            println!("connect error: {}", e);
        }
    }
}
