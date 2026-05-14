use socket2::{Domain, Protocol, Socket, Type};
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::time::{Duration, Instant};

const ICMP_ECHO_REPLY: u8 = 0;
const ICMP_DEST_UNREACH: u8 = 3;
const ICMP_ECHO_REQUEST: u8 = 8;
const ICMP_TTL_EXCEEDED: u8 = 11;

fn main() {
    let mut args = std::env::args().skip(1);
    let host = args.next().unwrap_or_else(|| {
        eprintln!("usage: icmp_traceroute <host> [probes] [max_hops] [timeout_ms]");
        std::process::exit(1);
    });
    let probes: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);
    let max_hops: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(30);
    let timeout_ms: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2000);

    let dest = (host.as_str(), 0u16)
        .to_socket_addrs()
        .ok()
        .and_then(|it| {
            it.filter_map(|s| match s.ip() {
                IpAddr::V4(v4) => Some(v4),
                _ => None,
            })
            .next()
        })
        .unwrap_or_else(|| {
            eprintln!("could not resolve {host} to IPv4");
            std::process::exit(1);
        });

    println!(
        "traceroute to {host} ({dest}), max {max_hops} hops, {probes} probes per hop, timeout {timeout_ms} ms"
    );

    let sock = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))
        .expect("raw socket failed (need root or CAP_NET_RAW)");

    let id = std::process::id() as u16;
    let mut seq: u16 = 0;

    for ttl in 1..=max_hops {
        sock.set_ttl(ttl as u32).expect("set_ttl failed");
        print!("{:2} ", ttl);

        let mut last_addr: Option<Ipv4Addr> = None;
        let mut reached = false;

        for _ in 0..probes {
            seq = seq.wrapping_add(1);
            let pkt = build_echo(id, seq);
            let dst = SocketAddr::new(IpAddr::V4(dest), 0);
            let start = Instant::now();
            if let Err(e) = sock.send_to(&pkt, &dst.into()) {
                print!(" send-err({e})");
                continue;
            }

            match recv_one(&sock, id, seq, Duration::from_millis(timeout_ms), start) {
                Some((from, kind, rtt)) => {
                    if last_addr != Some(from) {
                        if last_addr.is_some() {
                            print!("\n   ");
                        }
                        match reverse_lookup(from) {
                            Some(name) => print!(" {name} ({from})"),
                            None => print!(" {from}"),
                        }
                        last_addr = Some(from);
                    }
                    print!("  {:.2} ms", rtt.as_secs_f64() * 1000.0);
                    if kind == ICMP_ECHO_REPLY {
                        reached = true;
                    } else if kind == ICMP_DEST_UNREACH {
                        print!(" !H");
                        reached = true;
                    }
                }
                None => print!("  *"),
            }
        }
        println!();
        if reached {
            break;
        }
    }
}

fn build_echo(id: u16, seq: u16) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(16);
    pkt.push(ICMP_ECHO_REQUEST);
    pkt.push(0);
    pkt.extend_from_slice(&[0, 0]);
    pkt.extend_from_slice(&id.to_be_bytes());
    pkt.extend_from_slice(&seq.to_be_bytes());
    pkt.extend_from_slice(b"trace_rs");
    let cs = checksum(&pkt);
    pkt[2] = (cs >> 8) as u8;
    pkt[3] = (cs & 0xff) as u8;
    pkt
}

fn reverse_lookup(addr: Ipv4Addr) -> Option<String> {
    let octets = addr.octets();
    let sa = libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_port: 0,
        sin_addr: libc::in_addr {
            s_addr: u32::from_ne_bytes(octets),
        },
        sin_zero: [0; 8],
    };
    let mut host = [0 as libc::c_char; 256];
    let r = unsafe {
        libc::getnameinfo(
            &sa as *const _ as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
            host.as_mut_ptr(),
            host.len() as libc::socklen_t,
            std::ptr::null_mut(),
            0,
            libc::NI_NAMEREQD,
        )
    };
    if r != 0 {
        return None;
    }
    let cs = unsafe { CStr::from_ptr(host.as_ptr()) };
    cs.to_str().ok().map(|s| s.to_string())
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn recv_one(
    sock: &Socket,
    id: u16,
    want_seq: u16,
    timeout: Duration,
    start: Instant,
) -> Option<(Ipv4Addr, u8, Duration)> {
    let deadline = Instant::now() + timeout;
    let mut buf = [MaybeUninit::<u8>::uninit(); 1500];
    loop {
        let now = Instant::now();
        if now >= deadline {
            return None;
        }
        sock.set_read_timeout(Some(deadline - now)).ok();
        match sock.recv_from(&mut buf) {
            Ok((n, from)) => {
                let data: &[u8] =
                    unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                let from_ip = match from.as_socket() {
                    Some(SocketAddr::V4(v4)) => *v4.ip(),
                    _ => continue,
                };
                if data.len() < 20 {
                    continue;
                }
                let ihl = (data[0] & 0x0f) as usize * 4;
                if data.len() < ihl + 8 {
                    continue;
                }
                let icmp = &data[ihl..];
                let kind = icmp[0];

                if kind == ICMP_ECHO_REPLY {
                    let r_id = u16::from_be_bytes([icmp[4], icmp[5]]);
                    let r_seq = u16::from_be_bytes([icmp[6], icmp[7]]);
                    if r_id == id && r_seq == want_seq {
                        return Some((from_ip, kind, start.elapsed()));
                    }
                } else if kind == ICMP_TTL_EXCEEDED || kind == ICMP_DEST_UNREACH {
                    if icmp.len() < 8 + 20 {
                        continue;
                    }
                    let inner = &icmp[8..];
                    let inner_ihl = (inner[0] & 0x0f) as usize * 4;
                    if inner.len() < inner_ihl + 8 {
                        continue;
                    }
                    let inner_icmp = &inner[inner_ihl..];
                    if inner_icmp[0] != ICMP_ECHO_REQUEST {
                        continue;
                    }
                    let r_id = u16::from_be_bytes([inner_icmp[4], inner_icmp[5]]);
                    let r_seq = u16::from_be_bytes([inner_icmp[6], inner_icmp[7]]);
                    if r_id == id && r_seq == want_seq {
                        return Some((from_ip, kind, start.elapsed()));
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                return None;
            }
            Err(_) => return None,
        }
    }
}
