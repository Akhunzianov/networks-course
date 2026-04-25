use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use stop_and_wait::{checksum, verify_checksum};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

const LOSS_PROB: f64 = 0.1;
const HEADER_SIZE: usize = 6;
const MAX_DATA_SIZE: usize = 1018;
const PACKET_SIZE: usize = 1024;

fn should_drop() -> bool {
    rand::random::<f64>() < LOSS_PROB
}

async fn recv_file(rx: &mut mpsc::Receiver<Vec<u8>>, socket: &UdpSocket, src: SocketAddr) -> Vec<u8> {
    let mut file_data = Vec::new();
    let mut expected_seq: u8 = 0;

    loop {
        let packet = match rx.recv().await {
            Some(p) => p,
            None => { eprintln!("[{src}] channel closed"); return file_data; }
        };

        if should_drop() {
            println!("[{src}] drop  pkt");
            continue;
        }

        if packet.len() < HEADER_SIZE {
            eprintln!("[{src}] short packet, ignoring");
            continue;
        }

        let seq = packet[0];
        let is_last = packet[1] != 0;
        let data_len = u16::from_be_bytes([packet[2], packet[3]]) as usize;
        let cksum = u16::from_be_bytes([packet[4], packet[5]]);

        if HEADER_SIZE + data_len > packet.len() {
            eprintln!("[{src}] malformed packet, ignoring");
            continue;
        }

        let payload = &packet[HEADER_SIZE..HEADER_SIZE + data_len];

        if !verify_checksum(payload, cksum) {
            eprintln!("[{src}] checksum error seq={seq}, dropping");
            continue;
        }

        let ack = {
            let mut a = [0u8; HEADER_SIZE];
            a[0] = seq;
            a
        };

        let accepted = seq == expected_seq;
        if accepted {
            file_data.extend_from_slice(payload);
            expected_seq ^= 1;
            println!("[{src}] recv  seq={seq} ({data_len}B)");
        } else {
            println!("[{src}] dup   seq={seq}, re-ack");
        }

        if should_drop() {
            println!("[{src}] drop  ack={seq}");
        } else {
            let _ = socket.send_to(&ack, src).await;
            println!("[{src}] sent  ack={seq}");
        }

        if accepted && is_last {
            return file_data;
        }
    }
}

async fn send_file(
    rx: &mut mpsc::Receiver<Vec<u8>>,
    socket: &UdpSocket,
    peer: SocketAddr,
    data: &[u8],
    timeout_ms: u64,
) {
    let chunks: Vec<&[u8]> = data.chunks(MAX_DATA_SIZE).collect();
    let total = chunks.len();
    let mut seq: u8 = 0;

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == total - 1;
        let cksum = checksum(chunk);
        let mut packet = vec![0u8; HEADER_SIZE + chunk.len()];
        packet[0] = seq;
        packet[1] = is_last as u8;
        packet[2..4].copy_from_slice(&(chunk.len() as u16).to_be_bytes());
        packet[4..6].copy_from_slice(&cksum.to_be_bytes());
        packet[HEADER_SIZE..].copy_from_slice(chunk);

        loop {
            if should_drop() {
                println!("[{peer}] drop  seq={seq} chunk={}/{total}", i + 1);
            } else {
                match socket.send_to(&packet, peer).await {
                    Ok(_) => println!("[{peer}] sent  seq={seq} chunk={}/{total}", i + 1),
                    Err(e) => { eprintln!("[{peer}] send error: {e}"); continue; }
                }
            }

            match timeout(Duration::from_millis(timeout_ms), rx.recv()).await {
                Ok(Some(ack)) if !ack.is_empty() && ack[0] == seq => {
                    println!("[{peer}] ack   seq={seq}");
                    seq ^= 1;
                    break;
                }
                Ok(Some(_)) => println!("[{peer}] wrong ack, retransmit seq={seq}"),
                Ok(None) => { eprintln!("[{peer}] channel closed"); return; }
                Err(_) => println!("[{peer}] timeout, retransmit seq={seq}"),
            }
        }
    }

    println!("[{peer}] done  {total} chunks sent");
}

async fn handle_client(
    mut rx: mpsc::Receiver<Vec<u8>>,
    socket: Arc<UdpSocket>,
    src: SocketAddr,
    send_path: String,
    timeout_ms: u64,
) {
    println!("[{src}] connected");

    let data = recv_file(&mut rx, &socket, src).await;
    let out_path = format!("received_{}.bin", src.port());
    match std::fs::write(&out_path, &data) {
        Ok(_) => println!("[{src}] saved {out_path} ({} bytes)", data.len()),
        Err(e) => eprintln!("[{src}] write error: {e}"),
    }

    let send_data = match std::fs::read(&send_path) {
        Ok(d) => d,
        Err(e) => { eprintln!("[{src}] cannot read '{send_path}': {e}"); return; }
    };

    println!("[{src}] sending '{send_path}' back");
    send_file(&mut rx, &socket, src, &send_data, timeout_ms).await;
    println!("[{src}] disconnected");
}

type ClientMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>>;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let port = args.next().unwrap_or_else(|| "8080".to_string());
    let send_path = args.next().unwrap_or_else(|| "server_send.txt".to_string());
    let timeout_ms: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1000);

    let addr = format!("127.0.0.1:{port}");
    let socket = Arc::new(UdpSocket::bind(&addr).await.expect("failed to bind"));
    println!("listening on {addr}");

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let mut buf = [0u8; PACKET_SIZE];

    loop {
        let (len, src) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => { eprintln!("recv error: {e}"); continue; }
        };

        let packet = buf[..len].to_vec();
        let mut map = clients.lock().await;

        if let Some(tx) = map.get(&src) {
            if tx.send(packet).await.is_err() {
                map.remove(&src);
            }
        } else {
            let (tx, rx) = mpsc::channel(64);
            if tx.send(packet).await.is_ok() {
                map.insert(src, tx);
                let socket_clone = socket.clone();
                let clients_clone = clients.clone();
                let send_path_clone = send_path.clone();
                tokio::spawn(async move {
                    handle_client(rx, socket_clone, src, send_path_clone, timeout_ms).await;
                    clients_clone.lock().await.remove(&src);
                });
            }
        }
    }
}
