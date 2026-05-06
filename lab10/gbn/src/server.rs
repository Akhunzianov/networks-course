use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gbn::{PKT_ACK, PKT_DATA, PKT_FIN, PKT_FIN_ACK, decode, encode};

fn main() {
    let mut args = std::env::args().skip(1);
    let bind = args.next().unwrap_or_else(|| "127.0.0.1:9000".into());
    let out_path = args.next().unwrap_or_else(|| "received.bin".into());
    let loss: f64 = args
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let loss = if loss < 0.0 {
        0.0
    } else if loss > 1.0 {
        1.0
    } else {
        loss
    };

    let sock = UdpSocket::bind(&bind).expect("bind failed");
    sock.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
    println!("[server] listening on {bind}, file -> {out_path}, loss={loss}");

    let mut file = File::create(&out_path).expect("create failed");
    let mut expected: u32 = 0;
    let mut buf = [0u8; 4096];
    let mut recent_ooo: Vec<u32> = Vec::new();

    loop {
        let (n, src) = match sock.recv_from(&mut buf) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("[server] recv error: {e}");
                return;
            }
        };

        let (kind, seq, payload) = match decode(&buf[..n]) {
            Some(x) => x,
            None => {
                eprintln!("[server] bad packet");
                continue;
            }
        };

        if loss > 0.0 {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64;
            let r = ((nanos.wrapping_mul(2862933555777941757).wrapping_add(3037000493)) >> 33)
                as f64
                / (u32::MAX as f64);
            if r < loss {
                let k = match kind {
                    PKT_DATA => "DATA",
                    PKT_ACK => "ACK",
                    PKT_FIN => "FIN",
                    PKT_FIN_ACK => "FIN-ACK",
                    _ => "?",
                };
                println!("[server] <-- {k} seq={seq} (DROPPED, expected={expected})");
                continue;
            }
        }

        if kind == PKT_DATA {
            if seq == expected {
                file.write_all(payload).expect("write failed");
                println!(
                    "[server] <-- DATA seq={seq} ({} bytes) -> in-order, ACK {seq}",
                    payload.len()
                );
                expected += 1;
                recent_ooo.retain(|s| *s >= expected);
                let _ = sock.send_to(&encode(PKT_ACK, seq, &[]), src);
            } else if expected == 0 {
                println!("[server] <-- DATA seq={seq} (out-of-order, expected=0, no ACK yet)");
                if !recent_ooo.contains(&seq) {
                    recent_ooo.push(seq);
                }
            } else {
                let last = expected - 1;
                println!(
                    "[server] <-- DATA seq={seq} (out-of-order, expected={expected}) -> ACK {last}"
                );
                if !recent_ooo.contains(&seq) {
                    recent_ooo.push(seq);
                }
                let _ = sock.send_to(&encode(PKT_ACK, last, &[]), src);
            }
            recent_ooo.sort();
            if recent_ooo.len() > 8 {
                let drop_n = recent_ooo.len() - 8;
                recent_ooo.drain(0..drop_n);
            }
            let ooo_str: Vec<String> = recent_ooo.iter().map(|s| s.to_string()).collect();
            let last_acked: i64 = if expected == 0 { -1 } else { (expected - 1) as i64 };
            println!(
                "  [server-state] delivered=[0..{expected}) waiting_for={expected} last_ack={last_acked} ooo_seen=[{}]",
                ooo_str.join(",")
            );
        } else if kind == PKT_FIN {
            println!("[server] <-- FIN seq={seq} -> FIN-ACK");
            let _ = sock.send_to(&encode(PKT_FIN_ACK, seq, &[]), src);
            break;
        } else {
            println!("[server] <-- unexpected kind={kind} seq={seq}");
        }
    }

    file.flush().ok();
    println!("[server] done. {expected} packets accepted, written to {out_path}");
}
