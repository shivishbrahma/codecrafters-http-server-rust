#[allow(unused_imports)]
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let buf_reader = BufReader::new(&mut _stream);
                let req = buf_reader.lines().next().unwrap().unwrap();
                let res = match req.as_str() {
                    "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
                    _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                };
                _stream.write_all(res.as_bytes()).unwrap()
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
