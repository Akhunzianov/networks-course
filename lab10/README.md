## Go-Back-N (2)

Сделал только gbn

```bash
cd gbn
cargo run --bin gbn_server -- <bind addr (e.g. 127.0.0.1:8888)> <output file> [simulatied loss (e.g. 0.25)]
```

В другом — клиент:

```bash
cd gbn
cargo run --bin gbn_client -- <server addr (e.g. 127.0.0.1:8888)> <input file> [simulatied loss (e.g. 0.25)]
```
