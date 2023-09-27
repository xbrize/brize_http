extern crate native_tls;

use self::native_tls::TlsStream;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

#[derive(Debug)]
pub struct Header {
    response_status: String,
    content_type: String,
    raw: String,
}

#[derive(Debug)]
pub struct Response {
    pub header: Header,
    pub body: String,
}

pub fn parse_response(mut stream: TlsStream<TcpStream>) -> Response {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut buf = String::new();

    let mut header = Header {
        response_status: String::new(),
        content_type: String::new(),
        raw: String::new(),
    };

    // Response Status Code
    buf_reader
        .read_line(&mut buf)
        .expect("Failed To Read Buffer Line");
    header
        .response_status
        .push_str(&buf.trim_end_matches("\r\n"));

    // Read lines in header
    loop {
        buf.clear();
        match buf_reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                if buf == "\r\n" {
                    buf.clear();
                    break;
                };
                let line = buf.to_ascii_lowercase();
                if line.starts_with("content-type:") {
                    header.content_type.push_str(&buf.trim_end_matches("\r\n"));
                };
                header.raw.push_str(&buf.trim_end_matches("\r\n"));
            }
            Err(_) => println!("Failed to read header line"),
        }
    }

    let mut body_stream = String::new();
    loop {
        buf.clear();
        match buf_reader.read_line(&mut buf) {
            Ok(0) => {
                body_stream.push_str(&buf.trim());
                break;
            }
            Ok(_) => {
                if &buf == "" {
                    break;
                };
                body_stream.push_str(&buf.trim());
            }
            Err(_) => break,
        };
        if &buf == "" {
            break;
        };
    }

    // Clean up JSON string. Remove size integers before and after { }
    let mut res_bytes = body_stream.into_bytes();

    while res_bytes[0] != 123 {
        res_bytes.remove(0);
    }

    while res_bytes[res_bytes.len() - 1] != 125 {
        res_bytes.remove(res_bytes.len() - 1);
    }

    Response {
        header: header,
        body: String::from_utf8(res_bytes).unwrap(),
    }
}
