use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::fs;
use std::env;

struct FtpClient {
    control: TcpStream,
    reader: BufReader<TcpStream>,
}

impl FtpClient {
    fn connect(host: &str, port: u16, user: &str, password: &str) -> io::Result<Self> {
        let control = TcpStream::connect((host, port))?;
        let reader = BufReader::new(control.try_clone()?);
        let mut client = FtpClient { control, reader };

        client.read_response()?;

        client.send_command(&format!("USER {}", user))?;
        client.read_response()?;

        client.send_command(&format!("PASS {}", password))?;
        let resp = client.read_response()?;

        if !resp.starts_with("230") && !resp.starts_with("200") {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Login failed"));
        }

        client.send_command("TYPE I")?;
        let _ = client.read_response()?;

        Ok(client)
    }

    fn send_command(&mut self, cmd: &str) -> io::Result<()> {
        write!(self.control, "{}\r\n", cmd)?;
        self.control.flush()
    }

    fn read_response(&mut self) -> io::Result<String> {
        let mut full = String::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            full.push_str(&line);
            let code = &line[..3.min(line.len())];
            if line.len() >= 4 && &line[3..4] == " " {
                break;
            }
            if line.len() < 4 {
                break;
            }
            let _ = code;
        }
        Ok(full)
    }

    fn pasv(&mut self) -> io::Result<TcpStream> {
        self.send_command("EPSV")?;
        let resp = self.read_response()?;

        let start = resp.find("|||").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No ||| in EPSV"))?;
        let rest = &resp[start+3..];
        let end = rest.find('|').ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No closing | in EPSV"))?;
        let port: u16 = rest[..end].trim().parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Bad port in EPSV"))?;

        let peer = self.control.peer_addr()?;
        let addr = format!("{}:{}", peer.ip(), port);

        TcpStream::connect(addr)
    }

    pub fn list(&mut self, path: &str) -> io::Result<()> {
        let mut data_conn = self.pasv()?;

        let cmd = if path.is_empty() {
            "LIST".to_string()
        } else {
            format!("LIST {}", path)
        };
        self.send_command(&cmd)?;
        let resp = self.read_response()?;

        if !resp.starts_with("125") && !resp.starts_with("150") {
            return Err(io::Error::new(io::ErrorKind::Other, "LIST failed"));
        }

        let mut listing = String::new();
        data_conn.read_to_string(&mut listing)?;
        drop(data_conn);

        self.read_response()?;

        if listing.is_empty() {
            println!("(empty directory)");
        } else {
            for line in listing.lines() {
                println!("{}", line);
            }
        }

        Ok(())
    }

    pub fn download(&mut self, remote_path: &str, local_path: &str) -> io::Result<()> {
        let mut data_conn = self.pasv()?;

        self.send_command(&format!("RETR {}", remote_path))?;
        let resp = self.read_response()?;

        if !resp.starts_with("125") && !resp.starts_with("150") {
            return Err(io::Error::new(io::ErrorKind::Other, format!("RETR failed: {}", resp.trim())));
        }

        let mut buf = Vec::new();
        data_conn.read_to_end(&mut buf)?;
        drop(data_conn);

        fs::write(local_path, &buf)?;
        println!("Saved {} bytes to '{}'", buf.len(), local_path);

        Ok(())
    }

    pub fn upload(&mut self, local_path: &str, remote_path: &str) -> io::Result<()> {
        let data = fs::read(local_path)?;
        println!("Read {} bytes from '{}'", data.len(), local_path);

        let mut data_conn = self.pasv()?;

        self.send_command(&format!("STOR {}", remote_path))?;
        let resp = self.read_response()?;

        if !resp.starts_with("125") && !resp.starts_with("150") {
            return Err(io::Error::new(io::ErrorKind::Other, format!("STOR failed: {}", resp.trim())));
        }

        data_conn.write_all(&data)?;
        data_conn.flush()?;
        drop(data_conn);

        println!("Upload complete: '{}' → '{}'", local_path, remote_path);
        Ok(())
    }

    pub fn quit(&mut self) -> io::Result<()> {
        self.send_command("QUIT")?;
        let _resp = self.read_response()?;
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Incorrect usage");
        std::process::exit(1);
    }

    let cmd = &args[1];

    match cmd.as_str() {
        "list" => {
            if args.len() < 7 {
                eprintln!("Incorrect usage");
                std::process::exit(1);
            }
            let (host, port, user, pass, path) = (
                &args[2], args[3].parse::<u16>().unwrap_or(21),
                &args[4], &args[5], &args[6],
            );
            let mut client = FtpClient::connect(host, port, user, pass)
                .expect("Failed to connect");
            client.list(path).expect("LIST failed");
            client.quit().ok();
        }

        "get" => {
            if args.len() < 8 {
                eprintln!("Incorrect usage");
                std::process::exit(1);
            }
            let (host, port, user, pass, remote, local) = (
                &args[2], args[3].parse::<u16>().unwrap_or(21),
                &args[4], &args[5], &args[6], &args[7],
            );
            let mut client = FtpClient::connect(host, port, user, pass)
                .expect("Failed to connect");
            client.download(remote, local).expect("RETR failed");
            client.quit().ok();
        }

        "put" => {
            if args.len() < 8 {
                eprintln!("Incorrect usage");
                std::process::exit(1);
            }
            let (host, port, user, pass, local, remote) = (
                &args[2], args[3].parse::<u16>().unwrap_or(21),
                &args[4], &args[5], &args[6], &args[7],
            );
            let mut client = FtpClient::connect(host, port, user, pass)
                .expect("Failed to connect");
            client.upload(local, remote).expect("STOR failed");
            client.quit().ok();
        }

        _ => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    }
}
