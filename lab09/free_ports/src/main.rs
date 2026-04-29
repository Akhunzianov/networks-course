use std::net::{IpAddr, SocketAddr, TcpListener};

fn main() {
    let mut args = std::env::args().skip(1);
    let ip: IpAddr = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("usage: free_ports <ip> <from> <to>");
            std::process::exit(1);
        });
    let from: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        eprintln!("usage: free_ports <ip> <from> <to>");
        std::process::exit(1);
    });
    let to: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        eprintln!("usage: free_ports <ip> <from> <to>");
        std::process::exit(1);
    });

    if from > to {
        eprintln!("'from' must be <= 'to'");
        std::process::exit(1);
    }

    println!("free TCP ports from {ip}:{from} to {to}");
    let mut count = 0;
    for port in from..=to {
        if is_free(SocketAddr::new(ip, port)) {
            println!("{port}");
            count += 1;
        }
    }
    println!("total free: {count}");
}

fn is_free(addr: SocketAddr) -> bool {
    TcpListener::bind(addr).is_ok()
}
