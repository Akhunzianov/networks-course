## Трассировка маршрута с использованием ICMP

```bash
cd icmp_traceroute
cargo build --release
sudo ./target/release/icmp_traceroute <host> [probes (default 3)] [max_hops (default 30)] [timeout_ms (default 2000)]
```

Пример:

```bash
sudo ./target/release/icmp_traceroute 8.8.8.8
sudo ./target/release/icmp_traceroute yandex.ru 5 30 1500
```

## Использование протокола IPv6

```bash
cd echov6
cargo run --bin echov6_server -- <port>
```

```bash
cd echov6
cargo run --bin echov6_client -- <host> <port>
```

Пример:

```bash
cargo run --bin echov6_server -- 8888
cargo run --bin echov6_client -- ::1 8888 
```
