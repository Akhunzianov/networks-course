use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const INFINITY: u32 = 16;
const TICK: Duration = Duration::from_millis(200);
const IDLE_TICKS_TO_STOP: u32 = 5;

type Vector = Vec<(Ipv4Addr, u32)>;
type Message = (Ipv4Addr, Vector);
type Table = BTreeMap<Ipv4Addr, (Ipv4Addr, u32)>;

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let extra: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(n / 2);

    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;

    let ips: Vec<Ipv4Addr> = (0..n).map(|_| random_ip(&mut seed)).collect();
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];

    for i in 1..n {
        link(&mut neighbors, i, (rand(&mut seed) as usize) % i);
    }
    for _ in 0..extra {
        let a = (rand(&mut seed) as usize) % n;
        let b = (rand(&mut seed) as usize) % n;
        if a != b {
            link(&mut neighbors, a, b);
        }
    }

    for i in 0..n {
        let ns: Vec<String> = neighbors[i].iter().map(|j| ips[*j].to_string()).collect();
        println!("{} -> [{}]", ips[i], ns.join(", "));
    }
    println!();

    let mut senders: Vec<Sender<Message>> = Vec::with_capacity(n);
    let mut receivers: Vec<Option<Receiver<Message>>> = Vec::with_capacity(n);
    for _ in 0..n {
        let (tx, rx) = channel();
        senders.push(tx);
        receivers.push(Some(rx));
    }

    let stdout = Mutex::new(());
    let stdout = std::sync::Arc::new(stdout);

    thread::scope(|s| {
        for i in 0..n {
            let ip = ips[i];
            let nbr_ips: Vec<Ipv4Addr> = neighbors[i].iter().map(|j| ips[*j]).collect();
            let nbr_tx: Vec<Sender<Message>> = neighbors[i].iter().map(|j| senders[*j].clone()).collect();
            let rx = receivers[i].take().unwrap();
            let stdout = stdout.clone();
            s.spawn(move || run_router(ip, nbr_ips, nbr_tx, rx, stdout));
        }
    });
}

fn run_router(
    ip: Ipv4Addr,
    neighbor_ips: Vec<Ipv4Addr>,
    neighbor_tx: Vec<Sender<Message>>,
    rx: Receiver<Message>,
    stdout: std::sync::Arc<Mutex<()>>,
) {
    let mut table: Table = BTreeMap::new();
    table.insert(ip, (ip, 0));
    for n_ip in &neighbor_ips {
        table.insert(*n_ip, (*n_ip, 1));
    }

    let broadcast = |table: &Table| {
        let v: Vector = table.iter().map(|(d, (_, m))| (*d, *m)).collect();
        for tx in &neighbor_tx {
            let _ = tx.send((ip, v.clone()));
        }
    };
    broadcast(&table);

    let mut idle = 0u32;
    while idle < IDLE_TICKS_TO_STOP {
        let deadline = std::time::Instant::now() + TICK;
        let mut changed = false;
        loop {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            match rx.recv_timeout(deadline - now) {
                Ok((from, vector)) => {
                    for (dest, m) in vector {
                        if dest == ip {
                            continue;
                        }
                        let new_metric = (m + 1).min(INFINITY);
                        let better = match table.get(&dest) {
                            Some((_, cur)) => new_metric < *cur,
                            None => true,
                        };
                        if better {
                            table.insert(dest, (from, new_metric));
                            changed = true;
                        }
                    }
                }
                Err(_) => break,
            }
        }
        if changed {
            broadcast(&table);
            idle = 0;
        } else {
            idle += 1;
        }
    }

    let _g = stdout.lock().unwrap();
    println!("Final state of router {ip} table:");
    println!(
        "{:<16} {:<19} {:<16} {:>8}",
        "[Source IP]", "[Destination IP]", "[Next Hop]", "[Metric]"
    );
    for (dest, (next, metric)) in &table {
        if *dest == ip {
            continue;
        }
        println!(
            "{:<16} {:<19} {:<16} {:>8}",
            ip.to_string(),
            dest.to_string(),
            next.to_string(),
            metric
        );
    }
    println!();
}

fn link(neighbors: &mut [Vec<usize>], a: usize, b: usize) {
    if !neighbors[a].contains(&b) {
        neighbors[a].push(b);
    }
    if !neighbors[b].contains(&a) {
        neighbors[b].push(a);
    }
}

fn random_ip(seed: &mut u64) -> Ipv4Addr {
    loop {
        let a = (rand(seed) % 223 + 1) as u8;
        let b = (rand(seed) % 256) as u8;
        let c = (rand(seed) % 256) as u8;
        let d = (rand(seed) % 254 + 1) as u8;
        if a != 127 && a != 10 {
            return Ipv4Addr::new(a, b, c, d);
        }
    }
}

fn rand(seed: &mut u64) -> u32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 32) as u32
}
