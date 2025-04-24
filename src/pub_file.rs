use std::fs::File;
use std::io::{Read, Write};

pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn write_file(path: &str, contents: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())
}
