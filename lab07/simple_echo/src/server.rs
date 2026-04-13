use std::net::UdpSocket;
use std::env;

use rand::Rng;

fn main() {
    let port = env::args().nth(1).expect("pass port as arg");
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("failed binding socket");
    let mut buf = [0u8; 65536];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if rand::thread_rng().gen_range(0..10) < 2 {
                    continue;
                }
                let msg = String::from_utf8_lossy(&buf[..len]);
                let response = msg.to_uppercase();
                socket.send_to(response.as_bytes(), src).expect("failed to send");
            }
            Err(e) => {
                eprintln!("Error receiving: {}", e);
            }
        }
    }
}
