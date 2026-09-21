# Stage 1: Builder
FROM rust:1.93-slim AS builder

WORKDIR /build

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for prebuild dependencies (speeds up repeated builds)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual code
COPY src ./src

# Compile with optimization and strip to reduce size
RUN cargo build --release && \
    strip target/release/md-check

# Stage 2: Runtime (minimal image)
FROM debian:stable-slim

WORKDIR /app

# TLS is handled by rustls (no OpenSSL needed), but root certificates are
# loaded from the system store via rustls-native-certs
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /build/target/release/md-check /usr/local/bin/md-check

ENTRYPOINT ["/usr/local/bin/md-check"]
CMD []
