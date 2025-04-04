use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use flate2::{write::GzEncoder, Compression};

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
    request_status: HTTPStatus,
    request_content_type: HTTPContentType,
    response_content: String,
    request_headers: HashMap<String, String>,
) -> String {
    let mut content = response_content.clone();

    let response_enc_type = match request_headers.get("accept-encoding") {
        Some(enc_type) => {
            if enc_type.contains("gzip") {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                println!("{}", content);
                let _ = encoder.write_all(content.as_bytes());
                content = encoder
                    .finish()
                    .unwrap()
                    .iter()
                    .map(|&x| x as char)
                    .collect::<String>();
                String::from("\r\nContent-Encoding: gzip")
            } else {
                String::new()
            }
        }
        None => String::new(),
    };

    format!(
        "HTTP/1.1 {}\r\n{}",
        status(request_status),
        if content.len() > 0 {
            format!(
                "Content-Type: {}{}\r\nContent-Length: {}\r\n\r\n{}",
                content_type(request_content_type),
                response_enc_type,
                content.len(),
                content
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
    let http_request_type = request_parts[0].trim().to_lowercase();
    let http_request_path = request_parts[1].trim().to_lowercase();

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

    let mut http_request_body = String::new();
    match headers.get("content-length") {
        Some(content_length_str) => {
            if content_length_str != "0" {
                let content_length = content_length_str.parse::<usize>().unwrap();
                let mut buf = vec![0; content_length];
                let bytes_read = buf_reader.read(&mut buf).unwrap();
                if bytes_read != content_length {
                    eprintln!("Error: Client sent fewer bytes than specified in Content-Length");
                    return; // Close the connection
                }
                http_request_body = buf.iter().map(|&x| x as char).collect::<String>();
            }
        }
        None => {}
    }

    let response = if http_request_type == "get" {
        if http_request_path == "/" {
            build_response(
                HTTPStatus::Ok,
                HTTPContentType::TextPlain,
                String::new(),
                headers,
            )
        } else if http_request_path.starts_with("/echo") {
            let echo_str = http_request_path.trim_start_matches("/echo/");
            build_response(
                HTTPStatus::Ok,
                HTTPContentType::TextPlain,
                echo_str.to_string(),
                headers,
            )
        } else if http_request_path.starts_with("/user-agent") {
            let user_agent = headers.get("user-agent").unwrap();
            build_response(
                HTTPStatus::Ok,
                HTTPContentType::TextPlain,
                user_agent.to_string(),
                headers,
            )
        } else if http_request_path.starts_with("/files") {
            let file_path = file_dir + http_request_path.trim_start_matches("/files/");
            match read_file(file_path) {
                Ok(contents) => build_response(
                    HTTPStatus::Ok,
                    HTTPContentType::ApplicationOctetStream,
                    contents,
                    headers,
                ),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    build_response(
                        HTTPStatus::NotFound,
                        HTTPContentType::TextPlain,
                        String::new(),
                        headers,
                    )
                }
            }
        } else {
            build_response(
                HTTPStatus::NotFound,
                HTTPContentType::TextPlain,
                String::new(),
                headers,
            )
        }
    } else if http_request_type == "post" {
        if http_request_path.starts_with("/files") {
            let file_path = file_dir + http_request_path.trim_start_matches("/files/");
            match write_file(file_path, http_request_body) {
                Ok(_) => build_response(
                    HTTPStatus::Created,
                    HTTPContentType::ApplicationOctetStream,
                    String::new(),
                    headers,
                ),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    build_response(
                        HTTPStatus::NotFound,
                        HTTPContentType::TextPlain,
                        String::new(),
                        headers,
                    )
                }
            }
        } else {
            build_response(
                HTTPStatus::NotFound,
                HTTPContentType::TextPlain,
                String::new(),
                headers,
            )
        }
    } else {
        build_response(
            HTTPStatus::MethodNotAllowed,
            HTTPContentType::TextPlain,
            String::new(),
            headers,
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
                eprintln!("Error: {}", e);
            }
        }
    }
}
