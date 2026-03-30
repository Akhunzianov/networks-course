use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: client <host:port> <command>");
        std::process::exit(1);
    }
    let addr = &args[1];
    let command = args[2..].join(" ");
    let mut stream = TcpStream::connect(addr).expect("Failed to connect");

    stream.write_all(format!("{command}\n").as_bytes()).unwrap();
    stream.flush().unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    print!("{response}");
}
