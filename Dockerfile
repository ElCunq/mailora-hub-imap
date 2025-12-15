# Build stage
FROM rust:1.81-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update --allow-releaseinfo-change && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a dummy project to cache dependencies
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    echo "" > src/lib.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .
# Touch main.rs to force rebuild
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/mailora-hub-imap /usr/local/bin/mailora-hub-imap

# Copy static assets and migrations
COPY static /app/static
COPY migrations /app/migrations

# Environment variables
ENV PORT=3030
ENV DATABASE_URL=sqlite:///data/mailora_imap.db
ENV RUST_LOG=info,mailora_hub_imap=debug

# Create data directory
RUN mkdir -p /data

EXPOSE 3030

CMD ["mailora-hub-imap"]
