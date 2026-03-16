use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use httparse;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) => return,
        Ok(n) => n,
        Err(e) => {
            eprintln!("Read failed: {}", e);
            return;
        }
    };

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    if let Err(e) = req.parse(&buffer[..bytes_read]) {
        eprintln!("Parse failed: {}", e);
        return;
    }

    let path = req.path.unwrap_or("/");
    let filepath = match urlencoding::decode(path.trim_start_matches('/')) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Decode failed: {}", e);
            return;
        }
    };

    let response = match fs::read_to_string(filepath.as_ref()) {
        Ok(body) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        ),
        Err(e) => {
            let msg = e.to_string();
            format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                msg.len(),
                msg
            )
        }
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Write failed: {}", e);
    }
}

fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| "3000".to_string());
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
