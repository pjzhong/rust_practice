use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(listener) => listener,
        Err(err) => {
            println!("can't not bind {}", err);
            return;
        }
    };

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        } else {
            println!("Something went wrong {:?}", stream);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let prefix = b"*";
    if let Ok(size) = stream.read(&mut buffer) {
        println!("read {} bytes", size);
        if buffer.starts_with(prefix) {
            if let Ok(s) = String::from_utf8(buffer[1..size].to_vec()) {
                let mut s = s.split("\r\n");
                s.next();
                s.next();
                let command = s.next().unwrap_or("none");
                if command == "ping" {
                    s.next();
                    if let Some(str) = s.next() {
                        stream.write_all(str.as_bytes());
                    } else {
                        stream.write_all("pong".as_bytes());
                    }
                }
            }
        }
    }

    if let Err(e) = stream.shutdown(Shutdown::Both) {
        println!("{:?} shutdown wrong {:?}, do nothing", stream, e);
    }
}
