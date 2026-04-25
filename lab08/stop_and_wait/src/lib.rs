use std::net::{SocketAddr, UdpSocket};

const LOSS_PROB: f64 = 0.1;
pub const HEADER_SIZE: usize = 6;
pub const MAX_DATA_SIZE: usize = 1018;
const PACKET_SIZE: usize = 1024;

fn should_drop() -> bool {
    rand::random::<f64>() < LOSS_PROB
}

pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

pub fn verify_checksum(data: &[u8], cksum: u16) -> bool {
    checksum(data) == cksum
}

fn build_packet(seq: u8, is_last: bool, chunk: &[u8]) -> Vec<u8> {
    let cksum = checksum(chunk);
    let mut packet = vec![0u8; HEADER_SIZE + chunk.len()];
    packet[0] = seq;
    packet[1] = is_last as u8;
    packet[2..4].copy_from_slice(&(chunk.len() as u16).to_be_bytes());
    packet[4..6].copy_from_slice(&cksum.to_be_bytes());
    packet[HEADER_SIZE..].copy_from_slice(chunk);
    packet
}

pub fn send_file(socket: &UdpSocket, peer: &str, data: &[u8]) {
    let chunks: Vec<&[u8]> = data.chunks(MAX_DATA_SIZE).collect();
    let total = chunks.len();
    let mut seq: u8 = 0;

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == total - 1;
        let packet = build_packet(seq, is_last, chunk);

        loop {
            if should_drop() {
                println!("drop  seq={seq} chunk={}/{total}", i + 1);
            } else {
                match socket.send_to(&packet, peer) {
                    Ok(_) => println!("sent  seq={seq} chunk={}/{total}", i + 1),
                    Err(e) => { eprintln!("send error: {e}"); continue; }
                }
            }

            let mut ack_buf = [0u8; HEADER_SIZE];
            match socket.recv_from(&mut ack_buf) {
                Ok(_) if ack_buf[0] == seq => {
                    println!("ack   seq={seq}");
                    seq ^= 1;
                    break;
                }
                Ok(_) => println!("wrong ack, retransmit seq={seq}"),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                       || e.kind() == std::io::ErrorKind::TimedOut => {
                    println!("timeout, retransmit seq={seq}");
                }
                Err(e) => eprintln!("recv error: {e}"),
            }
        }
    }

    println!("done  {total} chunks sent");
}

pub fn recv_file(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
    let mut buf = [0u8; PACKET_SIZE];
    let mut file_data: Vec<u8> = Vec::new();
    let mut expected_seq: u8 = 0;

    loop {
        let (len, src) = match socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e) => { eprintln!("recv error: {e}"); continue; }
        };

        if should_drop() {
            println!("drop  pkt from {src}");
            continue;
        }

        if len < HEADER_SIZE {
            eprintln!("short packet ({len}B), ignoring");
            continue;
        }

        let seq = buf[0];
        let is_last = buf[1] != 0;
        let data_len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
        let cksum = u16::from_be_bytes([buf[4], buf[5]]);

        if HEADER_SIZE + data_len > len {
            eprintln!("malformed packet, ignoring");
            continue;
        }

        let payload = &buf[HEADER_SIZE..HEADER_SIZE + data_len];

        if !verify_checksum(payload, cksum) {
            eprintln!("checksum error seq={seq}, dropping");
            continue;
        }

        let ack = {
            let mut a = [0u8; HEADER_SIZE];
            a[0] = seq;
            a
        };

        if seq == expected_seq {
            file_data.extend_from_slice(payload);
            expected_seq ^= 1;
            println!("recv  seq={seq} ({data_len}B)");

            if should_drop() {
                println!("drop  ack={seq}");
            } else if let Err(e) = socket.send_to(&ack, src) {
                eprintln!("send ack error: {e}");
            } else {
                println!("sent  ack={seq}");
            }

            if is_last {
                return (file_data, src);
            }
        } else {
            println!("dup   seq={seq}, re-ack");
            if should_drop() {
                println!("drop  ack={seq}");
            } else if let Err(e) = socket.send_to(&ack, src) {
                eprintln!("send ack error: {e}");
            } else {
                println!("sent  ack={seq}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_correct_data() {
        let data = b"hello world";
        let cksum = checksum(data);
        assert!(verify_checksum(data, cksum));
    }

    #[test]
    fn checksum_detects_bit_flip() {
        let data = b"hello world";
        let cksum = checksum(data);
        let mut corrupted = data.to_vec();
        corrupted[3] ^= 0b0000_1000;
        assert!(!verify_checksum(&corrupted, cksum));
    }

    #[test]
    fn checksum_empty_data() {
        let data = b"";
        let cksum = checksum(data);
        assert!(verify_checksum(data, cksum));
    }
}
