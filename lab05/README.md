# A.1 

## Run

**Plain text:**
```bash
cargo run -- --to <recipient@example.com> --from <sender@example.com> \
  --smtp-host <host-name> --smtp-port <host-port> \
  --username <YOUR_USERNAME> --password <YOUR_PASSWORD> \
  --format txt --subject "Hello" --body "Plain text message"
```

**HTML:**
```bash
cargo run -- --to <recipient@example.com> --from <sender@example.com> \
  --smtp-host <host-name> --smtp-port <host-port> \
  --username <YOUR_USERNAME> --password <YOUR_PASSWORD> \
  --format html --subject "Hello" --body "<h1>Hello</h1><p>HTML message</p>"
```

# A.2 & A.3

## Run

**Plain text:**
```bash
cargo run -- --to <recipient@example.com> --from <sender@example.com> \
  --smtp-host <host-name> --smtp-port <host-port> \
  --username <YOUR_USERNAME> --password <YOUR_PASSWORD> \
  --subject "Hello" --body "Plain text message"
```

**Image:**
```bash
cargo run -- --to <recipient@example.com> --from <sender@example.com> \
  --smtp-host <host-name> --smtp-port <host-port> \
  --username <YOUR_USERNAME> --password <YOUR_PASSWORD> \
  --subject "Hello" --body "With image" --attachment </path/to/image>
```

**No TLS:**
```bash
cargo run -- --to <recipient@example.com> --from <sender@example.com> \
  --smtp-host mail.spbu.ru --smtp-port 25 --no-starttls \
  --subject "Hello" --body "Plain text message"
```

# Б

## Run

**Server:**
```bash
cargo run --bin server -- <port>
```

**Client:**
```bash
cargo run --bin client -- <sevrer-address (e.g. localhost:8000)> "<your shell cmd (e.g. ping -c 3 yandex.ru)>"
```
