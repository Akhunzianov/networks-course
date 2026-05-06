use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use gbn::{MAX_PAYLOAD, PKT_ACK, PKT_DATA, PKT_FIN, PKT_FIN_ACK, decode, encode};

fn main() {
    let mut args = std::env::args().skip(1);
    let server = args.next().unwrap_or_else(|| "127.0.0.1:9000".into());
    let path = args.next().unwrap_or_else(|| {
        eprintln!("usage: gbn_client <server> <file> [loss]");
        std::process::exit(1);
    });
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

    let mut data = Vec::new();
    File::open(&path)
        .expect("open failed")
        .read_to_end(&mut data)
        .expect("read failed");
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let end = (i + MAX_PAYLOAD).min(data.len());
        chunks.push(data[i..end].to_vec());
        i = end;
    }
    let total = chunks.len() as u32;
    println!(
        "[client] sending {} ({} bytes, {} packets) to {}, loss={}",
        path,
        data.len(),
        total,
        server,
        loss
    );

    let sock = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
    sock.connect(&server).expect("connect failed");
    sock.set_read_timeout(Some(Duration::from_millis(20))).unwrap();

    let window_size: usize = 4;
    let timeout = Duration::from_millis(500);
    let mut base: u32 = 0;
    let mut next_seq: u32 = 0;
    let mut window: VecDeque<u32> = VecDeque::new();
    let mut timer: Option<Instant> = None;
    let mut buf = [0u8; 4096];

    while base < total {
        while next_seq < total && window.len() < window_size {
            let pkt = encode(PKT_DATA, next_seq, &chunks[next_seq as usize]);
            let mut drop = false;
            if loss > 0.0 {
                let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos()
                    as u64;
                let r = ((nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
                    >> 33) as f64
                    / (u32::MAX as f64);
                if r < loss {
                    drop = true;
                }
            }
            if drop {
                println!("[client] --> DATA seq={next_seq} (LOSS-SIMULATED, not sent)");
            } else {
                let _ = sock.send(&pkt);
                println!(
                    "[client] --> DATA seq={next_seq} ({} bytes)",
                    chunks[next_seq as usize].len()
                );
            }
            window.push_back(next_seq);
            if timer.is_none() {
                timer = Some(Instant::now());
            }
            next_seq += 1;
            let inflight: Vec<String> = window.iter().map(|s| s.to_string()).collect();
            let acked = if base == 0 {
                "[]".to_string()
            } else {
                format!("[0..{base})")
            };
            let not_sent = if next_seq >= total {
                "[]".to_string()
            } else {
                format!("[{next_seq}..{total})")
            };
            println!(
                "  [client-state] base={base} next={next_seq} total={total} acked={acked} in-flight=[{}] not-sent={not_sent}",
                inflight.join(",")
            );
        }

        match sock.recv(&mut buf) {
            Ok(n) => {
                if let Some((kind, seq, _)) = decode(&buf[..n]) {
                    if kind == PKT_ACK {
                        if seq < total && seq + 1 > base {
                            let new_base = seq + 1;
                            println!(
                                "[client] <-- ACK {seq}  (window slides: base {base} -> {new_base})"
                            );
                            base = new_base;
                            while let Some(&f) = window.front() {
                                if f < base {
                                    window.pop_front();
                                } else {
                                    break;
                                }
                            }
                            timer = if window.is_empty() {
                                None
                            } else {
                                Some(Instant::now())
                            };
                            let inflight: Vec<String> =
                                window.iter().map(|s| s.to_string()).collect();
                            let acked = if base == 0 {
                                "[]".to_string()
                            } else {
                                format!("[0..{base})")
                            };
                            let not_sent = if next_seq >= total {
                                "[]".to_string()
                            } else {
                                format!("[{next_seq}..{total})")
                            };
                            println!(
                                "  [client-state] base={base} next={next_seq} total={total} acked={acked} in-flight=[{}] not-sent={not_sent}",
                                inflight.join(",")
                            );
                        } else {
                            println!("[client] <-- ACK {seq}  (duplicate, base={base})");
                        }
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("[client] recv error: {e}");
                return;
            }
        }

        if let Some(t) = timer {
            if t.elapsed() >= timeout && !window.is_empty() {
                println!(
                    "[client] !! timeout, retransmit window [{}..{})",
                    base,
                    base + window.len() as u32
                );
                for &seq in window.iter() {
                    let pkt = encode(PKT_DATA, seq, &chunks[seq as usize]);
                    let mut drop = false;
                    if loss > 0.0 {
                        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64;
                        let r = ((nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
                            >> 33) as f64
                            / (u32::MAX as f64);
                        if r < loss {
                            drop = true;
                        }
                    }
                    if drop {
                        println!("[client] --> DATA seq={seq} (re, LOSS-SIMULATED)");
                    } else {
                        let _ = sock.send(&pkt);
                        println!("[client] --> DATA seq={seq} (re)");
                    }
                }
                timer = Some(Instant::now());
            }
        }
    }

    println!("[client] all data acked, sending FIN");
    let fin = encode(PKT_FIN, total, &[]);
    let mut tries = 0;
    while tries < 10 {
        let _ = sock.send(&fin);
        if let Ok(n) = sock.recv(&mut buf) {
            if let Some((kind, _, _)) = decode(&buf[..n]) {
                if kind == PKT_FIN_ACK {
                    println!("[client] <-- FIN-ACK, done");
                    return;
                }
            }
        }
        tries += 1;
    }
    println!("[client] no FIN-ACK, exiting");
}
