use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use std::time::Duration;
use stop_and_wait::{recv_file, send_file};

fn main() {
    let mut args = std::env::args().skip(1);
    let port = args.next().unwrap_or_else(|| "8080".to_string());
    let file_path = args.next().unwrap_or_else(|| "send_file.txt".to_string());
    let timeout_ms: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1000);

    let server_addr = format!("127.0.0.1:{port}");

    let file_data = fs::read(&file_path).unwrap_or_else(|e| {
        eprintln!("cannot read '{file_path}': {e}");
        std::process::exit(1);
    });

    let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind");
    socket.set_read_timeout(Some(Duration::from_millis(timeout_ms))).expect("set_read_timeout failed");

    println!("sending '{file_path}' → {server_addr} ({} bytes)", file_data.len());
    send_file(&socket, &server_addr, &file_data);

    println!("waiting for file from server...");
    socket.set_read_timeout(None).expect("set_read_timeout failed");
    let (data, _) = recv_file(&socket);
    match File::create("client_received.bin").and_then(|mut f| f.write_all(&data)) {
        Ok(_) => println!("saved client_received.bin ({} bytes)", data.len()),
        Err(e) => eprintln!("write error: {e}"),
    }
}
