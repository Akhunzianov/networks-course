use std::net::UdpSocket;
use std::env;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use std::sync::{Arc, Mutex};

struct ClientHeart {
    heartbeat_time: Instant,
    heartbeat_seq: u64,
    lost: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        std::process::exit(1);
    }
    let port = &args[1];
    let dead_time: u64 = args[2].parse().expect("failed parsing time to die");
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("failed binding socket");

    let clients_dict: Arc<Mutex<HashMap<String, ClientHeart>>> = Arc::new(Mutex::new(HashMap::new()));
    let clients_heartbeater = Arc::clone(&clients_dict);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut dict = clients_heartbeater.lock().unwrap();
            let mut to_remove = Vec::new();
            for (addr, heart) in dict.iter() {
                if heart.heartbeat_time.elapsed().as_secs() >= dead_time {
                    println!("Client {} died", addr);
                    to_remove.push(addr.clone());
                }
            }
            for addr in to_remove {
                dict.remove(&addr);
            }
        }
    });

    let mut buf = [0u8; 65536];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                let parts: Vec<&str> = msg.trim().splitn(2, ' ').collect();
                if parts.len() != 2 {
                    continue;
                }
                let seq: u64 = match parts[0].parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let client_t: f64 = match parts[1].parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let cur_t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
                let delay = cur_t - client_t;
                let addr = src.to_string();
                let mut dict = clients_dict.lock().unwrap();
                let heart = dict.entry(addr.clone()).or_insert(ClientHeart {
                    heartbeat_time: Instant::now(),
                    heartbeat_seq: 0,
                    lost: 0,
                });
                if heart.heartbeat_seq > 0 && seq > heart.heartbeat_seq + 1 {
                    let missed = seq - heart.heartbeat_seq - 1;
                    heart.lost += missed;
                    println!("Client {} LOST {} packet(s) (seq {} -> {})", addr, missed, heart.heartbeat_seq, seq);
                }
                heart.heartbeat_time = Instant::now();
                heart.heartbeat_seq = seq;
                println!("Client {} seq={} delay={:.3}ms lost_total={}", addr, seq, delay * 1000.0, heart.lost);
            }
            Err(_) => {}
        }
    }
}
