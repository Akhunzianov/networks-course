## IP-адрес и маска сети (1)

```bash
cd ip_netmask
cargo run
```

## Доступные порты (2)

```bash
cd free_ports
cargo run -- <ip> <from> <to>
```

Пример:

```bash
cargo run -- 127.0.0.1 8080 8090
```

## Подсчет копий приложения (3)

```bash
cd copies
cargo run
```

Надо запустить несколько копий в разных терминалах
