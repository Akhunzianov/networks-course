use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let mut args = env::args().skip(1);
    let host = args.next().expect("no host");
    let port = args.next().expect("no port");
    let file = args.next().expect("no file");

    let addr = format!("{}:{}", host, port);

    let mut stream = TcpStream::connect(&addr).expect("Failed to connect to server");
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        file,
        host
    );
    stream.write_all(request.as_bytes()).expect("Failed to send request");

    let mut response = String::new();
    stream.read_to_string(&mut response)
        .expect("Failed to read response");

    println!("{}", response);
}
