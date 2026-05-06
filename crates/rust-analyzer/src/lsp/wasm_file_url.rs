use std::path::{Path, PathBuf};

use lsp_types::Url;

pub trait WasmFileUrlExt {
    fn from_file_path<P: AsRef<Path>>(path: P) -> Result<Url, ()>;
    fn to_file_path(&self) -> Result<PathBuf, ()>;
}

impl WasmFileUrlExt for Url {
    fn from_file_path<P: AsRef<Path>>(path: P) -> Result<Url, ()> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(());
        }

        let path = path.to_str().ok_or(())?;
        Url::parse(&format!("file://{}", encode_file_path(path))).map_err(|_| ())
    }

    fn to_file_path(&self) -> Result<PathBuf, ()> {
        match self.host_str() {
            None | Some("") | Some("localhost") => (),
            Some(_) => return Err(()),
        }

        let path = decode_file_path(self.path()).ok_or(())?;
        let path = PathBuf::from(path);
        path.is_absolute().then_some(path).ok_or(())
    }
}

fn encode_file_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'/' | b':' | b'-' | b'.' | b'_' | b'~' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push('%');
                encoded.push(hex(byte >> 4));
                encoded.push(hex(byte & 0x0f));
            }
        }
    }
    encoded
}

fn decode_file_path(path: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(path.len());
    let mut iter = path.bytes();
    while let Some(byte) = iter.next() {
        if byte != b'%' {
            bytes.push(byte);
            continue;
        }

        let high = unhex(iter.next()?)?;
        let low = unhex(iter.next()?)?;
        bytes.push((high << 4) | low);
    }

    String::from_utf8(bytes).ok()
}

fn hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + nibble - 10) as char,
        _ => unreachable!(),
    }
}

fn unhex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
