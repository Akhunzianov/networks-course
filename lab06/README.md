## Run ftp_client list

```bash
cd ftp_client
cargo run -- list <HOST> <PORT> <USER> <PASSWORD> <REMOTE_PATH>
```

## Run ftp_client get

```bash
cd ftp_client
cargo run -- get <HOST> <PORT> <USER> <PASSWORD> <REMOTE_FILE_PATH> <LOCAL_FILE_PATH>
```

## Run ftp_client put

```bash
cd ftp_client
cargo run -- put <HOST> <PORT> <USER> <PASSWORD> <LOCAL_FILE_PATH> <REMOTE_FILE_PATH>
```
