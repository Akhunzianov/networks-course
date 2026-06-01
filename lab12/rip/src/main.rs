use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};

const INFINITY: u32 = 16;

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

    let mut tables: Vec<Table> = (0..n)
        .map(|i| {
            let mut t = Table::new();
            t.insert(ips[i], (ips[i], 0));
            for j in &neighbors[i] {
                t.insert(ips[*j], (ips[*j], 1));
            }
            t
        })
        .collect();

    let mut step = 0;
    loop {
        step += 1;
        let snapshot = tables.clone();
        let mut changed = false;
        for i in 0..n {
            for j in &neighbors[i] {
                let from = ips[*j];
                for (dest, (_, m)) in &snapshot[*j] {
                    if *dest == ips[i] {
                        continue;
                    }
                    let new_metric = (m + 1).min(INFINITY);
                    let better = match tables[i].get(dest) {
                        Some((_, cur)) => new_metric < *cur,
                        None => true,
                    };
                    if better {
                        tables[i].insert(*dest, (from, new_metric));
                        changed = true;
                    }
                }
            }
        }
        for i in 0..n {
            print_table(&format!("Simulation step {step} of router {}", ips[i]), ips[i], &tables[i]);
            println!();
        }
        if !changed {
            break;
        }
    }

    for i in 0..n {
        print_table(&format!("Final state of router {} table:", ips[i]), ips[i], &tables[i]);
        println!();
    }
}

fn print_table(header: &str, src: Ipv4Addr, table: &Table) {
    println!("{header}");
    println!(
        "{:<16} {:<19} {:<16} {:>8}",
        "[Source IP]", "[Destination IP]", "[Next Hop]", "[Metric]"
    );
    for (dest, (next, metric)) in table {
        if *dest == src {
            continue;
        }
        println!(
            "{:<16} {:<19} {:<16} {:>8}",
            src.to_string(),
            dest.to_string(),
            next.to_string(),
            metric
        );
    }
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
