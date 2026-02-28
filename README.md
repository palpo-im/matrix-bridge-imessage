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

## Quick Start

### Using Docker

```bash
# Clone the repository
git clone https://github.com/palpo-im/matrix-bridge-imessage.git
cd matrix-bridge-imessage

# Copy and edit configuration
cp config/config.sample.yaml config/config.yaml
# Edit config.yaml with your settings

# Start with Docker Compose
docker-compose up -d
```

### From Source

```bash
# Clone the repository
git clone https://github.com/palpo-im/matrix-bridge-imessage.git
cd matrix-bridge-imessage

# Build
cargo build --release

# Copy and edit configuration
cp config/config.sample.yaml config/config.yaml
# Edit config.yaml with your settings

# Run
./target/release/matrix-bridge-imessage -c config/config.yaml
```

## Configuration

### Basic Configuration

```yaml
bridge:
  domain: "your-homeserver.com"
  homeserver_url: "https://matrix.your-homeserver.com"
  port: 9006
  appservice_token: "YOUR_AS_TOKEN"
  homeserver_token: "YOUR_HS_TOKEN"

platform:
  platform: "bluebubbles"
  bluebubbles_url: "http://localhost:1234"
  bluebubbles_password: "YOUR_BLUEBUBBLES_PASSWORD"

database:
  url: "sqlite://./imessage.db"
```

### Platform-Specific Configuration

#### BlueBubbles

1. Install BlueBubbles on your macOS device
2. Configure the server settings in BlueBubbles
3. Get your server URL and password
4. Add them to the configuration

#### mac-nosip

Requires macOS with SIP disabled and Barcelona installed.

## Matrix Setup

1. Generate a registration file:
   ```bash
   ./matrix-bridge-imessage -c config.yaml --generate-registration
   ```

2. Add the registration file to your homeserver configuration

3. Restart your homeserver

4. Start the bridge

## Usage

### Bridging a Chat

1. Start a conversation with the bridge bot
2. Use the `!imessage bridge` command to bridge a chat

### Commands

- `!imessage help` - Show available commands
- `!imessage bridge <chat-id>` - Bridge an iMessage chat
- `!imessage unbridge` - Unbridge current room

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

Apache-2.0

## Acknowledgments

- [mautrix-imessage](https://github.com/mautrix/imessage) - Reference implementation
- [matrix-bridge-discord](https://github.com/palpo-im/matrix-bridge-discord) - Project structure reference
