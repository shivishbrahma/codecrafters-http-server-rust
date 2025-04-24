mod pub_file;
mod pub_http;
mod pub_server;

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                std::thread::spawn(move || {
                    let mut buffer = vec![0; 1024];
                    while let Ok(bytes_read) = _stream.read(&mut buffer) {
                        if bytes_read == 0 {
                            break;
                        }
                        let (response, close) = pub_server::handle_request(&buffer[..bytes_read]);
                        _stream.write_all(&response).unwrap();
                        if close {
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
