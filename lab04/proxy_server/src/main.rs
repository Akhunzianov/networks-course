use std::env;
use std::fs;
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use httparse;
use threadpool::ThreadPool;
use reqwest;

macro_rules! http_error {
    ($status:expr, $msg:expr) => {
        format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            $status,
            $msg.len(),
            $msg
        )
    };
}

static LOG_FILE: std::sync::OnceLock<Mutex<std::fs::File>> = std::sync::OnceLock::new();
static BLACKLIST: std::sync::OnceLock<HashSet<String>> = std::sync::OnceLock::new();

fn init_log() {
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("proxy.log")
        .unwrap();
    LOG_FILE.set(Mutex::new(file)).unwrap();
}

fn init_blacklist(path: &str) {
    let entries = fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect::<HashSet<String>>();
    BLACKLIST.set(entries).unwrap();
}

fn check_blacklist(url: &str) -> bool {
    let blacklist = match BLACKLIST.get() {
        Some(b) => b,
        None => return false,
    };
    let stripped = url
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    for entry in blacklist {
        if stripped == entry.as_str() || stripped.starts_with(&format!("{}/", entry)) {
            return true;
        }
    }
    false
}

macro_rules! log {
    ($($arg:tt)*) => {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let msg = format!($($arg)*);
        if let Some(mutex) = LOG_FILE.get() {
            let mut file = mutex.lock().unwrap();
            writeln!(file, "[{}] {}", ts, msg).unwrap();
        }
    }
}

#[derive(Debug)]
struct CacheEntry {
    status: u16,
    reason: String,
    etag: Option<String>,
    filename: PathBuf,
    expiration: std::time::Instant,
}

static COUNTER: AtomicU64 = AtomicU64::new(0);
static CACHE: std::sync::OnceLock<Mutex<HashMap<String, CacheEntry>>> = std::sync::OnceLock::new();

fn init_cache() {
    CACHE.set(Mutex::new(HashMap::new())).unwrap();
}

fn put_to_cache(body: &str) -> String {
    let file_number = COUNTER.fetch_add(1, Ordering::Relaxed).to_string();
    let file_name = format!("cache/{}.txt", file_number);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_name)
        .unwrap();
    write!(file, "{}", body).unwrap();
    file_name
}

fn get_from_cache(filename: &PathBuf) -> Option<String> {
    fs::read_to_string(filename).ok()
}

fn serve_url(
    method: &str,
    path: &str,
    headers: Vec<(String, String)>,
    body: &[u8],
) -> String {
    let host = path.trim_start_matches('/');
    if host.is_empty() {
        log!("Error No host url!");
        return http_error!("400 Bad Request", "No requested host");
    }

    let url = format!("http://{}", host);
    let cachable = method == "GET";

    if check_blacklist(&url) {
        let msg = format!("{} is in the blacklist", url);
        log!("<< [proxy->user] 403 Forbidden");
        log!("<<   body: {}", msg);
        return format!(
            "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            msg.len(),
            msg
        );
    }

    if cachable {
        let cache = CACHE.get().unwrap().lock().unwrap();
        if let Some(entry) = cache.get(&url) {
            if entry.expiration > std::time::Instant::now() {
                if let Some(body) = get_from_cache(&entry.filename) {
                    log!("<< [proxy->user] {} {} (cached)", entry.status, entry.reason);
                    log!("<<   body: {} bytes", body.len());
                    return format!(
                        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        entry.status, entry.reason, body.len(), body
                    );
                }
            }
        }
    }

    let client = reqwest::blocking::Client::new();
    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap(), &url);
    for (name, value) in &headers {
        req = req.header(name, value);
    }

    {
        let cache = CACHE.get().unwrap().lock().unwrap();
        if let Some(entry) = cache.get(&url) {
            if let Some(etag) = &entry.etag {
                req = req.header("If-None-Match", etag);
            }
        }
    }

    log!(">> [proxy->server] {} {}", method, url);

    let result = if body.is_empty() {
        req.send()
    } else {
        req.body(body.to_vec()).send()
    };

    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            let reason = response.status().canonical_reason().unwrap_or("Unknown");

            log!("<< [server->proxy] {} {}", status, reason);
            for (name, value) in response.headers() {
                log!("<<   {}: {}", name, value.to_str().unwrap_or("?"));
            }

            let etag = response.headers().get("etag").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let max_age = response.headers()
                .get("cache-control")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(',').find(|s| s.trim().starts_with("max-age=")))
                .and_then(|s| s.trim().trim_start_matches("max-age=").parse::<u64>().ok())
                .unwrap_or(60);

            if status == 304 {
                let mut cache = CACHE.get().unwrap().lock().unwrap();
                if let Some(entry) = cache.get_mut(&url) {
                    if let Some(body) = get_from_cache(&entry.filename) {
                        entry.expiration = std::time::Instant::now() + std::time::Duration::from_secs(max_age);
                        log!("<< [proxy->user] {} {} (revalidated)", entry.status, entry.reason);
                        return format!(
                            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            entry.status, entry.reason, body.len(), body
                        );
                    }
                }
            }

            let content_type = response.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let base_host = host.split('/').next().unwrap_or("");
            let body = if content_type.contains("text/html") {
                response.text().unwrap_or_default()
                    .replace("href=\"/", &format!("href=\"/{}/", base_host))
                    .replace("src=\"/", &format!("src=\"/{}/", base_host))
            } else {
                response.text().unwrap_or_default()
            };

            if cachable {
                let mut cache = CACHE.get().unwrap().lock().unwrap();
                let cached_file_name = put_to_cache(&body);
                cache.insert(url.clone(), CacheEntry {
                    status,
                    reason: reason.to_string(),
                    etag,
                    filename: cached_file_name.into(),
                    expiration: std::time::Instant::now() + std::time::Duration::from_secs(max_age),
                });
            }

            log!("<< [proxy->user] {} {} body: {} bytes", status, reason, body.len());

            format!(
                "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                reason,
                body.len(),
                body
            )
        }
        Err(e) => {
            log!("<< [proxy->user] 502 Bad Gateway: {}", e);
            http_error!("502 Bad Gateway", &format!("Failed: {}", e))
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = vec![0u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) => return,
        Ok(n) => n,
        Err(_) => return,
    };

    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);

    let header_len = match req.parse(&buffer[..bytes_read]) {
        Ok(httparse::Status::Complete(n)) => n,
        _ => return,
    };
    let method = req.method.unwrap_or("GET").to_string();
    let path = req.path.unwrap_or("/").to_string();

    let content_length = req.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case("content-length"))
        .and_then(|h| std::str::from_utf8(h.value).ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let owned_headers: Vec<(String, String)> = req.headers.iter()
        .map(|h| (
            h.name.to_string(),
            std::str::from_utf8(h.value).unwrap_or("").to_string(),
        ))
        .collect();

    log!(">> [user->proxy] {} {}", method, path);
    for (name, value) in &owned_headers {
        log!(">>   {}: {}", name, value);
    }
    if content_length > 0 {
        log!(">>   body: {} bytes", content_length);
    }

    drop(req);

    let body_in_buffer = bytes_read - header_len;
    let remaining = content_length.saturating_sub(body_in_buffer);
    if remaining > 0 {
        buffer.resize(bytes_read + remaining, 0);
        if stream.read_exact(&mut buffer[bytes_read..]).is_err() {
            return;
        }
    }

    let body = buffer[header_len..header_len + content_length].to_vec();

    let response = serve_url(&method, &path, owned_headers, &body);

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Write failed: {}", e);
    }
}

fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| "3000".to_string());
    let blacklist_path = env::args().nth(2).unwrap_or_else(|| "blacklist.conf".to_string());
    let concurrency_level: usize = env::args().nth(3).unwrap_or_else(|| "1".to_string()).parse().unwrap_or(1);
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).unwrap();
    let thread_pool = ThreadPool::new(concurrency_level);

    init_log();
    init_cache();
    init_blacklist(&blacklist_path);
    fs::create_dir_all("cache").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread_pool.execute(move || {
                    handle_client(stream);
                })
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
