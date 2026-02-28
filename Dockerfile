FROM rust:1.75 as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY config ./config

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/matrix-bridge-imessage /usr/local/bin/

# Copy default config
COPY config/config.sample.yaml /app/config/config.yaml

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 9006

ENTRYPOINT ["matrix-bridge-imessage"]
CMD ["-c", "/app/config/config.yaml"]
