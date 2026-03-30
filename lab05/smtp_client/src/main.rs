use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use native_tls::TlsConnector;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "SMTP client")]
struct Args {
    #[arg(short, long)]
    to: String,

    #[arg(short, long, default_value = "Test")]
    subject: String,

    #[arg(short, long, default_value = "Hello world!")]
    body: String,

    #[arg(long, default_value = "smtp.gmail.com")]
    smtp_host: String,

    #[arg(long, default_value_t = 587)]
    smtp_port: u16,

    #[arg(long, default_value = "sender@example.com")]
    from: String,

    #[arg(long)]
    username: Option<String>,

    #[arg(long)]
    password: Option<String>,

    #[arg(long, default_value_t = true)]
    starttls: bool,

    #[arg(long)]
    attachment: Option<PathBuf>,
}

fn read_response<R: BufRead>(reader: &mut R) -> String {
    let mut response = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read");
        let is_last = line.len() >= 4 && line.as_bytes()[3] == b' ';
        response.push_str(&line);
        if is_last {
            break;
        }
    }
    response
}

fn expect_code(response: &str, code: &str) {
    if !response.starts_with(code) {
        panic!("Expected {code}, got: {response}");
    }
}

fn cmd<W: Write>(w: &mut W, s: &str) {
    w.write_all(format!("{s}\r\n").as_bytes()).unwrap();
    w.flush().unwrap();
}

fn build_message(from: &str, to: &str, subject: &str, body: &str, attachment: Option<&PathBuf>) -> String {
    match attachment {
        None => {
            format!("From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\n\r\n{body}\r\n")
        }
        Some(path) => {
            let boundary = "----=_boundary_abc123";
            let filename = path.file_name().unwrap().to_string_lossy();
            let file_bytes = std::fs::read(path).expect("Failed to read attachment");
            let encoded = STANDARD.encode(&file_bytes);

            let encoded_lines = encoded
                .as_bytes()
                .chunks(76)
                .map(|c| std::str::from_utf8(c).unwrap())
                .collect::<Vec<_>>()
                .join("\r\n");

            format!(
                "From: {from}\r\n\
                 To: {to}\r\n\
                 Subject: {subject}\r\n\
                 MIME-Version: 1.0\r\n\
                 Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\
                 \r\n\
                 --{boundary}\r\n\
                 Content-Type: text/plain\r\n\
                 \r\n\
                 {body}\r\n\
                 \r\n\
                 --{boundary}\r\n\
                 Content-Type: application/octet-stream\r\n\
                 Content-Transfer-Encoding: base64\r\n\
                 Content-Disposition: attachment; filename=\"{filename}\"\r\n\
                 \r\n\
                 {encoded_lines}\r\n\
                 \r\n\
                 --{boundary}--\r\n"
            )
        }
    }
}

fn send_mail<T: Write + std::io::Read>(
    stream: &mut BufReader<T>,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
    attachment: Option<&PathBuf>,
) {
    cmd(stream.get_mut(), &format!("MAIL FROM:<{from}>"));
    expect_code(&read_response(stream), "250");

    cmd(stream.get_mut(), &format!("RCPT TO:<{to}>"));
    expect_code(&read_response(stream), "250");

    cmd(stream.get_mut(), "DATA");
    expect_code(&read_response(stream), "354");

    let message = build_message(from, to, subject, body, attachment);
    stream.get_mut().write_all(message.as_bytes()).unwrap();
    stream.get_mut().write_all(b".\r\n").unwrap();
    stream.get_mut().flush().unwrap();
    expect_code(&read_response(stream), "250");

    cmd(stream.get_mut(), "QUIT");
    let _ = read_response(stream);
}

fn main() {
    let args = Args::parse();
    let addr = format!("{}:{}", args.smtp_host, args.smtp_port);

    let tcp = TcpStream::connect(&addr).expect("TCP connection failed");

    if args.starttls {
        let mut reader = BufReader::new(tcp);
        expect_code(&read_response(&mut reader), "220");
        cmd(reader.get_mut(), &format!("EHLO {}", args.smtp_host));
        expect_code(&read_response(&mut reader), "250");
        cmd(reader.get_mut(), "STARTTLS");
        expect_code(&read_response(&mut reader), "220");

        let connector = TlsConnector::new().unwrap();
        let tls = connector
            .connect(&args.smtp_host, reader.into_inner())
            .expect("TLS handshake failed");
        let mut reader = BufReader::new(tls);

        cmd(reader.get_mut(), &format!("EHLO {}", args.smtp_host));
        expect_code(&read_response(&mut reader), "250");

        if let (Some(user), Some(pass)) = (args.username, args.password) {
            cmd(reader.get_mut(), "AUTH LOGIN");
            expect_code(&read_response(&mut reader), "334");
            cmd(reader.get_mut(), &STANDARD.encode(&user));
            expect_code(&read_response(&mut reader), "334");
            cmd(reader.get_mut(), &STANDARD.encode(&pass));
            expect_code(&read_response(&mut reader), "235");
        }

        send_mail(&mut reader, &args.from, &args.to, &args.subject, &args.body, args.attachment.as_ref());
    } else {
        let mut reader = BufReader::new(tcp);
        expect_code(&read_response(&mut reader), "220");
        cmd(reader.get_mut(), &format!("EHLO {}", args.smtp_host));
        expect_code(&read_response(&mut reader), "250");
        send_mail(&mut reader, &args.from, &args.to, &args.subject, &args.body, args.attachment.as_ref());
    }
}
