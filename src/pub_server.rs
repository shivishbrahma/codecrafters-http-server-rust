use crate::pub_file::{read_file, write_file};
use crate::pub_http::{build_response, ContentType, RequestType, ResponseStatus};
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Read};

pub fn handle_request(buffer: &[u8]) -> (Vec<u8>, bool) {
    let base_dir = env::args()
        .nth(2)
        .filter(|_| env::args().nth(1) == Some("--directory".to_string()))
        .unwrap_or_else(|| "/tmp/".to_string());

    let mut reader = BufReader::new(buffer);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    let mut connection_close = false;

    if parts.len() < 2 {
        return (
            build_response(
                ResponseStatus::NotFound,
                ContentType::TextPlain,
                String::new(),
                &HashMap::new(),
            ),
            connection_close,
        );
    }

    let method: RequestType = match RequestType::from_string(parts[0]) {
        Some(m) => m,
        None => {
            return (
                build_response(
                    ResponseStatus::NotFound,
                    ContentType::TextPlain,
                    String::new(),
                    &HashMap::new(),
                ),
                connection_close,
            )
        }
    };

    let path = parts[1].to_lowercase();

    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.trim().split_once(":") {
            headers.insert(k.trim().to_lowercase(), v.trim().to_lowercase());
        }
    }

    let mut body = String::new();
    if let Some(len) = headers.get("content-length") {
        if let Ok(size) = len.parse::<usize>() {
            let mut buf = vec![0; size];
            reader.read_exact(&mut buf).unwrap();
            body = String::from_utf8_lossy(&buf).to_string();
        }
    }

    // Header: Connection close
    if headers
        .get("connection")
        .map(|c| c.contains("close"))
        .unwrap_or(false)
    {
        connection_close = true;
    }

    match method {
        RequestType::Get => match path.as_str() {
            "/" => (
                build_response(
                    ResponseStatus::Ok,
                    ContentType::TextPlain,
                    String::new(),
                    &headers,
                ),
                connection_close,
            ),
            p if p.starts_with("/echo/") => {
                let content = p.trim_start_matches("/echo/");
                (
                    build_response(
                        ResponseStatus::Ok,
                        ContentType::TextPlain,
                        content.to_string(),
                        &headers,
                    ),
                    connection_close,
                )
            }
            "/user-agent" => {
                let agent = headers.get("user-agent").cloned().unwrap_or_default();
                (
                    build_response(
                        ResponseStatus::Ok,
                        ContentType::TextPlain,
                        agent.to_string(),
                        &headers,
                    ),
                    connection_close,
                )
            }
            p if p.starts_with("/files/") => {
                let full_path = format!("{}{}", base_dir, &p[7..]);
                match read_file(&full_path) {
                    Ok(contents) => (
                        build_response(
                            ResponseStatus::Ok,
                            ContentType::ApplicationOctetStream,
                            contents,
                            &headers,
                        ),
                        connection_close,
                    ),
                    Err(_) => (
                        build_response(
                            ResponseStatus::NotFound,
                            ContentType::TextPlain,
                            String::new(),
                            &headers,
                        ),
                        connection_close,
                    ),
                }
            }
            _ => (
                build_response(
                    ResponseStatus::NotFound,
                    ContentType::TextPlain,
                    String::new(),
                    &headers,
                ),
                connection_close,
            ),
        },
        RequestType::Post if path.starts_with("/files/") => {
            let full_path = format!("{}{}", base_dir, &path[7..]);
            match write_file(&full_path, &body) {
                Ok(_) => (
                    build_response(
                        ResponseStatus::Created,
                        ContentType::ApplicationOctetStream,
                        String::new(),
                        &headers,
                    ),
                    connection_close,
                ),
                Err(_) => (
                    build_response(
                        ResponseStatus::NotFound,
                        ContentType::TextPlain,
                        String::new(),
                        &headers,
                    ),
                    connection_close,
                ),
            }
        }
        _ => (
            build_response(
                ResponseStatus::MethodNotAllowed,
                ContentType::TextPlain,
                String::new(),
                &headers,
            ),
            connection_close,
        ),
    }
}
