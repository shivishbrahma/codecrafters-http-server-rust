use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
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
    ApplicationOctetStream,
}

fn status(http_status: HTTPStatus) -> String {
    match http_status {
        HTTPStatus::Ok => "200 OK",
        HTTPStatus::NotFound => "404 Not Found",
        HTTPStatus::MethodNotAllowed => "405 Method Not Allowed",
    }
    .to_string()
}

fn content_type(http_content_type: HTTPContentType) -> String {
    match http_content_type {
        HTTPContentType::TextPlain => "text/plain",
        // HTTPContentType::TextHtml => "text/html",
        HTTPContentType::ApplicationOctetStream => "application/octet-stream",
    }
    .to_string()
}

fn build_response(
    http_status: HTTPStatus,
    contents: String,
    http_content_type: HTTPContentType,
) -> String {
    let length = contents.len();
    format!(
        "HTTP/1.1 {}\r\n{}",
        status(http_status),
        if length > 0 {
            format!(
                "Content-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                content_type(http_content_type),
                length,
                contents
            )
        } else {
            "\r\n".to_string()
        }
    )
}

fn read_file(file_path: String) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn handle_request(mut _stream: TcpStream) {
    let buf_reader = BufReader::new(&mut _stream);
    let mut lines = buf_reader.lines();

    let file_dir = if env::args().len() > 2 && env::args().nth(1).unwrap() == "--directory" {
        env::args().nth(2).unwrap()
    } else {
        "/tmp/".to_string()
    };

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
        } else if request_path.starts_with("/files") {
            let file_path = file_dir + request_path.trim_start_matches("/files/");
            match read_file(file_path) {
                Ok(contents) => build_response(
                    HTTPStatus::Ok,
                    contents,
                    HTTPContentType::ApplicationOctetStream,
                ),
                Err(e) => {
                    println!("Error: {}", e);
                    build_response(
                        HTTPStatus::NotFound,
                        "".to_string(),
                        HTTPContentType::TextPlain,
                    )
                }
            }
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
