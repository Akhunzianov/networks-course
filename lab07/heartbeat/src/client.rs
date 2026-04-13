use std::net::{ToSocketAddrs, UdpSocket};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        std::process::exit(1);
    }
    let host = &args[1];
    let port = &args[2];
    let notify_every_ms: u64 = args[3].parse().expect("failed parsing time to notify");
    let serv = format!("{}:{}", host, port);
    let server_addr = serv.to_socket_addrs()
        .expect("failed to resolve")
        .next()
        .expect("no address found");
    let bind_addr = if server_addr.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let socket = UdpSocket::bind(bind_addr).expect("failed binding socket");

    for i in 1u64.. {
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let msg = format!("{} {}.{:06}", i, t.as_secs(), t.subsec_micros());
        socket.send_to(msg.as_bytes(), server_addr).expect("failed to send");
        println!("Sent heartbeat seq={} at {}.{:06}", i, t.as_secs(), t.subsec_micros());
        thread::sleep(Duration::from_millis(notify_every_ms));
    }
}
