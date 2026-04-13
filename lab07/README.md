## Simple echo (А, Б, В) 

**Server:**
```bash
cd simple_echo 
cargo run --bin server -- <port>
```

**Client:**
```bash
cd simple_echo
cargo run --bin client -- <server-address (e.g. 127.0.0.1)> <server-port>
```

## HeartBeat (Г) 

**Server:**
```bash
cd heartbeat 
cargo run --bin server -- <port> <client considered dead after such amount of secs>
```

**Client:**
```bash
cd heartbeat
cargo run --bin client -- <server-address (e.g. 127.0.0.1)> <server-port> <send heartbeat interval (ms)>
```
