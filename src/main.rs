use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

enum HTTPStatus {
    Ok,
    Created,
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
        HTTPStatus::Created => "201 Created",
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

fn write_file(file_path: String, contents: String) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    let _ = file.write_all(contents.as_bytes());
    Ok(())
}

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>());
// }

fn handle_request(mut _stream: TcpStream) {
    let file_dir = if env::args().len() > 2 && env::args().nth(1).unwrap() == "--directory" {
        env::args().nth(2).unwrap()
    } else {
        "/tmp/".to_string()
    };

    let mut buf_reader = BufReader::new(&mut _stream);
    let mut request = String::new();
    buf_reader.read_line(&mut request).unwrap();

    let request_parts: Vec<&str> = request.trim().split(" ").collect();
    let request_type = request_parts[0].trim().to_lowercase();
    let request_path = request_parts[1].trim().to_lowercase();

    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        let bytes_read = buf_reader.read_line(&mut line).unwrap();
        if bytes_read == 0 || line.trim().is_empty() {
            break;
        }
        let parts: Vec<&str> = line.trim().split(": ").collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_lowercase(), parts[1].to_lowercase());
        }
    }

    let mut data = String::new();
    match headers.get("content-length") {
        Some(content_length) => {
            if content_length != "0" {
                let _ = buf_reader.read_line(&mut data).unwrap();
            }
        }
        None => {}
    }

    println!("{} - {}", request_path, request_type);

    let response = if request_type == "get" {
        if request_path == "/" {
            build_response(HTTPStatus::Ok, String::new(), HTTPContentType::TextPlain)
        } else if request_path.starts_with("/echo") {
            let echo_str = request_path.trim_start_matches("/echo/");
            build_response(
                HTTPStatus::Ok,
                echo_str.to_string(),
                HTTPContentType::TextPlain,
            )
        } else if request_path.starts_with("/user-agent") {
            let user_agent = headers.get("user-agent").unwrap();
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
                        String::new(),
                        HTTPContentType::TextPlain,
                    )
                }
            }
        } else {
            build_response(
                HTTPStatus::NotFound,
                String::new(),
                HTTPContentType::TextPlain,
            )
        }
    } else if request_type == "post" {
        if request_path.starts_with("/files") {
            let file_path = file_dir + request_path.trim_start_matches("/files/");
            let _ = buf_reader.read_line(&mut data).unwrap();
            println!("Data: {}", data);
            match write_file(file_path, data) {
                Ok(_) => build_response(
                    HTTPStatus::Created,
                    String::new(),
                    HTTPContentType::ApplicationOctetStream,
                ),
                Err(e) => {
                    println!("Error: {}", e);
                    build_response(
                        HTTPStatus::NotFound,
                        String::new(),
                        HTTPContentType::TextPlain,
                    )
                }
            }
        } else {
            build_response(
                HTTPStatus::NotFound,
                String::new(),
                HTTPContentType::TextPlain,
            )
        }
    } else {
        build_response(
            HTTPStatus::MethodNotAllowed,
            String::new(),
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
