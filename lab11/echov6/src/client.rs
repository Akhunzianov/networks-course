use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};

fn main() {
    let mut args = std::env::args().skip(1);
    let host = args.next().unwrap_or_else(|| {
        eprintln!("usage: echov6_client <host> <port>");
        std::process::exit(1);
    });
    let port: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        eprintln!("usage: echov6_client <host> <port>");
        std::process::exit(1);
    });

    let addr: SocketAddr = (host.as_str(), port)
        .to_socket_addrs()
        .ok()
        .and_then(|it| it.filter(|s| matches!(s.ip(), IpAddr::V6(_))).next())
        .unwrap_or_else(|| {
            eprintln!("could not resolve {host} to IPv6");
            std::process::exit(1);
        });

    let stream = TcpStream::connect(addr).expect("connect failed");
    let mut writer = stream.try_clone().expect("clone failed");
    let mut reader = BufReader::new(stream);
    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut reply = String::new();

    loop {
        line.clear();
        let n = stdin.lock().read_line(&mut line).expect("stdin read failed");
        if n == 0 {
            return;
        }
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.is_empty() {
            continue;
        }
        if writeln!(writer, "{trimmed}").is_err() {
            return;
        }
        reply.clear();
        match reader.read_line(&mut reply) {
            Ok(0) => return,
            Ok(_) => print!("{reply}"),
            Err(_) => return,
        }
    }
}
