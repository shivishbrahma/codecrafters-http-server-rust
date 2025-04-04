use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

enum HTTPStatus {
    Ok,
    NotFound,
    MethodNotAllowed,
}

enum HTTPContentType {
    TextPlain,
    // TextHtml,
}

fn build_response(
    http_status: HTTPStatus,
    contents: String,
    http_content_type: HTTPContentType,
) -> String {
    let length = contents.len();
    let status = match http_status {
        HTTPStatus::Ok => "200 OK",
        HTTPStatus::NotFound => "404 Not Found",
        HTTPStatus::MethodNotAllowed => "405 Method Not Allowed",
    };
    let content_type = match http_content_type {
        HTTPContentType::TextPlain => "text/plain",
        // HTTPContentType::TextHtml => "text/html",
    };

    format!(
        "HTTP/1.1 {}\r\n{}",
        status,
        if length > 0 {
            format!(
                "Content-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                content_type, length, contents
            )
        } else {
            "\r\n".to_string()
        }
    )
}

fn handle_request(mut _stream: TcpStream) {
    let buf_reader = BufReader::new(&mut _stream);
    let mut lines = buf_reader.lines();

    let request = lines.next().unwrap().unwrap();
    let request_type = request.split(" ").nth(0).unwrap();
    let request_path = request.split(" ").nth(1).unwrap();

    let headers = lines
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .map(|line| {
            (
                line.split(": ").nth(0).unwrap().to_string(),
                line.split(": ").nth(1).unwrap().to_string(),
            )
        })
        .collect::<HashMap<String, String>>();

    let response = if request_type == "GET" {
        if request_path == "/" {
            build_response(HTTPStatus::Ok, "".to_string(), HTTPContentType::TextPlain)
        } else if request_path.starts_with("/echo") {
            let echo_str = request_path.trim_start_matches("/echo/");
            build_response(
                HTTPStatus::Ok,
                echo_str.to_string(),
                HTTPContentType::TextPlain,
            )
        } else if request_path.starts_with("/user-agent") {
            let user_agent = headers.get("User-Agent").unwrap();
            build_response(
                HTTPStatus::Ok,
                user_agent.to_string(),
                HTTPContentType::TextPlain,
            )
        } else {
            build_response(
                HTTPStatus::NotFound,
                "".to_string(),
                HTTPContentType::TextPlain,
            )
        }
    } else {
        build_response(
            HTTPStatus::MethodNotAllowed,
            "".to_string(),
            HTTPContentType::TextPlain,
        )
    };
    _stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                // handle_request(_stream);
                std::thread::spawn(move || handle_request(_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
