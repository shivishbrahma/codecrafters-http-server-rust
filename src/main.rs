use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_request(mut _stream: TcpStream) {
    let buf_reader = BufReader::new(&mut _stream);
    let request = buf_reader.lines().next().unwrap().unwrap();

    // let request_type = request.split(" ").nth(0).unwrap();
    let request_path = request.split(" ").nth(1).unwrap();

    let res = if request_path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else if request_path.starts_with("/echo") {
        let echo_str = request_path.trim_start_matches("/echo/");
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            echo_str.len(),
            echo_str
        )
        .to_string()
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    };
    _stream.write_all(res.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_request(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
