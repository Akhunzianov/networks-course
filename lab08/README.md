## Stop-and-Wait (А, Б, В)

**Server:**
```bash
cd stop_and_wait
cargo run --bin server -- <port> <file-to-send-back> <timeout-ms>
```

**Client:**
```bash
cd stop_and_wait
cargo run --bin client -- <port> <file-to-send> <timeout-ms>
```

**Tests (checksum):**
```bash
cd stop_and_wait
cargo test
```
