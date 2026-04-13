use std::net::{ToSocketAddrs, UdpSocket};
use std::env;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        std::process::exit(1);
    }
    let host = &args[1];
    let port = &args[2];
    let serv = format!("{}:{}", host, port);
    let server_addr = serv.to_socket_addrs()
        .expect("failed to resolve")
        .next()
        .expect("no address found");
    let bind_addr = if server_addr.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let socket = UdpSocket::bind(bind_addr).expect("failed binding socket");
    socket.set_read_timeout(Some(Duration::from_secs(1))).expect("failed setting timeout");

    println!("PING {} ({})", host, serv);
    let mut rtts: Vec<f64> = Vec::new();
    let mut lost = 0u32;
    for i in 1..=10 {
        let time_print = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let msg = format!("Ping {} {}.{:06}", i, time_print.as_secs(), time_print.subsec_micros());
        let t0 = Instant::now();
        socket.send_to(msg.as_bytes(), &server_addr).expect("failed to send");

        let mut buf = [0u8; 65536];
        match socket.recv_from(&mut buf) {
            Ok((len, _)) => {
                let rtt = t0.elapsed().as_secs_f64() * 1000.0;
                let resp = String::from_utf8_lossy(&buf[..len]);
                println!("{} bytes: seq={} time={:.3} ms", len, i, rtt);
                rtts.push(rtt);
                let _ = resp;
            }
            Err(_) => {
                println!("Request timeout for seq {}", i);
                lost += 1;
            }
        }
    }

    let total = 10u32;
    let received = total - lost;
    let loss_pct = (lost as f64 / total as f64) * 100.0;
    println!("\n--- {} ping statistics ---", host);
    println!("{} packets transmitted, {} received, {:.0}% packet loss", total, received, loss_pct);
    if !rtts.is_empty() {
        let min = rtts.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = rtts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg = rtts.iter().sum::<f64>() / rtts.len() as f64;
        println!("round trip min/avg/max = {:.3}/{:.3}/{:.3} ms", min, avg, max);
    }
}
