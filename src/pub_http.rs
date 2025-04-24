use flate2::{write::GzEncoder, Compression};
use std::collections::HashMap;
use std::io::Write;

pub enum ResponseStatus {
    Ok,
    Created,
    NotFound,
    MethodNotAllowed,
}

impl ResponseStatus {
    pub fn to_string(&self) -> &str {
        match self {
            ResponseStatus::Ok => "200 OK",
            ResponseStatus::Created => "201 Created",
            ResponseStatus::NotFound => "404 Not Found",
            ResponseStatus::MethodNotAllowed => "405 Method Not Allowed",
        }
    }
}

pub enum ContentType {
    TextPlain,
    ApplicationOctetStream,
}

impl ContentType {
    pub fn to_string(&self) -> String {
        match self {
            ContentType::TextPlain => "text/plain",
            ContentType::ApplicationOctetStream => "application/octet-stream",
        }
        .to_string()
    }
}

pub enum RequestType {
    Get,
    Post,
}

impl RequestType {
    // pub fn to_string(&self) -> &str {
    //     match self {
    //         RequestType::Get => "GET",
    //         RequestType::Post => "POST",
    //     }
    // }

    pub fn from_string(s: &str) -> Option<RequestType> {
        match s.to_lowercase().as_str() {
            "get" => Some(RequestType::Get),
            "post" => Some(RequestType::Post),
            _ => None,
        }
    }
}

pub fn build_response(
    status: ResponseStatus,
    content_type: ContentType,
    body: String,
    headers: &HashMap<String, String>,
) -> Vec<u8> {
    let mut response = vec![];
    response.extend_from_slice(format!("HTTP/1.1 {}\r\n", status.to_string()).as_bytes());

    let mut resp_headers: HashMap<String, String> = HashMap::new();

    match headers.get("connection") {
        Some(k) => {
            resp_headers.insert(String::from("Connection"), k.to_string());
        }
        _ => {}
    }

    let content: Vec<u8> = match headers.get("accept-encoding") {
        Some(enc) if enc.contains("gzip") => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body.trim().as_bytes()).unwrap();
            resp_headers.insert(String::from("Content-Encoding"), String::from("gzip"));
            encoder.finish().unwrap()
        }
        _ => body.trim().as_bytes().to_vec(),
    };

    if !content.is_empty() {
        resp_headers.insert(String::from("Content-Type"), content_type.to_string());
        resp_headers.insert(String::from("Content-Length"), content.len().to_string());
    }

    for (header, value) in resp_headers {
        response.extend_from_slice(format!("{}: {}\r\n", header, value).as_bytes());
    }
    response.extend_from_slice(b"\r\n");
    response.extend_from_slice(content.as_slice());

    println!("=====\n{}\n=====\n", String::from_utf8_lossy(&response));

    response
}
