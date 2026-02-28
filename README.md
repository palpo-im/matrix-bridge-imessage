# Matrix-iMessage Bridge

A Matrix-iMessage puppeting bridge written in Rust.

## Features

- Bridge iMessage conversations to Matrix rooms
- Support for text messages, media files, and attachments
- Support for replies, reactions (tapbacks), and edits
- Read receipts and typing notifications
- Group chat support
- Backfilling message history

## Supported Platforms

This bridge supports multiple iMessage connector platforms:

- **mac**: Native macOS iMessage (requires SIP)
- **mac-nosip**: macOS without SIP (using Barcelona)
- **bluebubbles**: BlueBubbles server connector

## Configuration

See `config/config.sample.yaml` for configuration options.

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/matrix-bridge-imessage -c config.yaml
```

## License

Apache-2.0
