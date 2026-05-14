use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv6Addr, SocketAddrV6, TcpListener, TcpStream};

fn main() {
    let mut args = std::env::args().skip(1);
    let port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        eprintln!("usage: echov6_server <port>");
        std::process::exit(1);
    });

    let addr = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);
    let listener = TcpListener::bind(addr).expect("bind failed");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                std::thread::spawn(move || handle(s));
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}

fn handle(stream: TcpStream) {
    let mut writer = stream.try_clone().expect("clone failed");
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        match line {
            Ok(msg) => {
                if writeln!(writer, "{}", msg.to_uppercase()).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
