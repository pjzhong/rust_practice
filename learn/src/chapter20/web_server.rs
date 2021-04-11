use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

use crate::chapter20::thread_pool::ThreadPool;

pub fn run() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(listener) => listener,
        Err(err) => {
            println!("can't not bind {}", err);
            return;
        }
    };

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            pool.execute(|| {
                handle_connection(stream);
            });
        } else {
            println!("Something went wrong {:?}", stream);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(size) => {
            println!("read {} bytes", size);

            let get = b"GET / HTTP/1.1\r\n";

            if buffer.starts_with(get) {
                let contents = include_str!("hello.html");
                let response = format!(
                    "HTTP/1.0 200 Ok\r\ncontent-Length: {}\r\ncontent-type: text/html\r\n\r\n{}",
                    contents.len(),
                    contents
                );
                write(&mut stream, response);
            } else {
                let status_line = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                let contents = include_str!("404.html");
                let response = format!("{}{}", status_line, contents);
                write(&mut stream, response);
            }
        }
        Err(e) => {
            println!("{:?} read wrong {:?}, closing", stream, e);
            stream.shutdown(Shutdown::Both);
            return;
        }
    }
}

fn write(stream: &mut TcpStream, response: String) {
    match stream.write(response.as_bytes()) {
        Ok(size) => {
            println!("write {} bytes", size);
            stream.flush();
        }
        Err(e) => {
            println!("{:?} write wrong {:?}, do nothing", stream, e);
        }
    };
}
