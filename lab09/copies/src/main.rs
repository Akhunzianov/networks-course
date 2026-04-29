use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const BROADCAST_INTERVAL: Duration = Duration::from_secs(2);
const PEER_TIMEOUT: Duration = Duration::from_secs(7);
const DISCOVERY_PORT: u16 = 35353;

fn main() {
    let id = process::id();

    let listener = bind_shared(DISCOVERY_PORT).expect("failed to bind listener");
    listener
        .set_read_timeout(Some(BROADCAST_INTERVAL))
        .expect("set_read_timeout failed");

    let sender = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .expect("failed to bind sender");
    sender.set_broadcast(true).expect("set_broadcast failed");
    sender
        .set_read_timeout(Some(BROADCAST_INTERVAL))
        .expect("set_read_timeout failed");

    let local_port = sender.local_addr().unwrap().port();
    println!("[start] pid={id} port={local_port}");

    let peers: Arc<Mutex<HashMap<SocketAddr, Instant>>> = Arc::new(Mutex::new(HashMap::new()));
    let listener = Arc::new(listener);
    let sender = Arc::new(sender);
    let running = Arc::new(AtomicBool::new(true));

    {
        let running = Arc::clone(&running);
        let sender = Arc::clone(&sender);
        let peers = Arc::clone(&peers);
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
            let msg = format!("LEAVE {id}");
            broadcast(&sender, &msg);
            for addr in peers.lock().unwrap().keys() {
                let _ = sender.send_to(msg.as_bytes(), addr);
            }
        })
        .expect("failed to set ctrl-c handler");
    }

    spawn_recv_loop("listener", Arc::clone(&listener), Arc::clone(&sender),
                    Arc::clone(&peers), Arc::clone(&running), id);
    spawn_recv_loop("sender", Arc::clone(&sender), Arc::clone(&sender),
                    Arc::clone(&peers), Arc::clone(&running), id);

    broadcast(&sender, &format!("JOIN {id}"));
    thread::sleep(Duration::from_millis(300));

    while running.load(Ordering::SeqCst) {
        let msg = format!("PING {id}");
        broadcast(&sender, &msg);
        for addr in peers.lock().unwrap().keys() {
            let _ = sender.send_to(msg.as_bytes(), addr);
        }
        prune(&peers);
        print_state(&peers, local_port);
        thread::sleep(BROADCAST_INTERVAL);
    }

    println!("[stop] leave");
}

fn spawn_recv_loop(
    _name: &'static str,
    socket: Arc<UdpSocket>,
    reply_socket: Arc<UdpSocket>,
    peers: Arc<Mutex<HashMap<SocketAddr, Instant>>>,
    running: Arc<AtomicBool>,
    self_id: u32,
) {
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        while running.load(Ordering::SeqCst) {
            match socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                    handle_message(&reply_socket, &peers, src, &msg, self_id);
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    eprintln!("recv error: {e}");
                    break;
                }
            }
        }
    });
}

fn handle_message(
    sender: &UdpSocket,
    peers: &Mutex<HashMap<SocketAddr, Instant>>,
    src: SocketAddr,
    msg: &str,
    self_id: u32,
) {
    let mut parts = msg.split_whitespace();
    let kind = parts.next().unwrap_or("");
    let peer_id: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    if peer_id == self_id {
        return;
    }

    match kind {
        "JOIN" => {
            peers.lock().unwrap().insert(src, Instant::now());
            let _ = sender.send_to(format!("PING {self_id}").as_bytes(), src);
        }
        "PING" => {
            peers.lock().unwrap().insert(src, Instant::now());
        }
        "LEAVE" => {
            peers.lock().unwrap().remove(&src);
        }
        _ => {}
    }
}

fn broadcast(sender: &UdpSocket, msg: &str) {
    let target = SocketAddrV4::new(Ipv4Addr::BROADCAST, DISCOVERY_PORT);
    let _ = sender.send_to(msg.as_bytes(), target);
}

fn prune(peers: &Mutex<HashMap<SocketAddr, Instant>>) {
    let now = Instant::now();
    peers
        .lock()
        .unwrap()
        .retain(|_, last| now.duration_since(*last) < PEER_TIMEOUT);
}

fn print_state(peers: &Mutex<HashMap<SocketAddr, Instant>>, self_port: u16) {
    let peers = peers.lock().unwrap();
    let total = peers.len() + 1;
    println!("[copies={total}] self=:{self_port}");
    if peers.is_empty() {
        println!("  (no other peers)");
    } else {
        for (i, addr) in peers.keys().enumerate() {
            println!("  [{}] {addr}", i + 1);
        }
    }
}

fn bind_shared(port: u16) -> std::io::Result<UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    let addr: SocketAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into();
    socket.bind(&addr.into())?;
    Ok(socket.into())
}
