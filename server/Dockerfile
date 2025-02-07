# Stage 1: Build
FROM rust:1.82-slim as builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

# Install Cargo Chef
RUN cargo install cargo-chef

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Prepare the build plan
RUN cargo chef prepare --recipe-path recipe.json

# Copy source code
COPY src ./src
COPY database/migrations ./database/migrations

# Cook the build plan
RUN cargo chef cook --recipe-path recipe.json

# Build
RUN cargo build --release

# Stage 2: Run
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

# Copy binary and migrations
COPY --from=builder /usr/src/app/target/release/swolemate-server ./
COPY --from=builder /usr/src/app/database/migrations ./database/migrations

# Create necessary directories
RUN mkdir -p database logs

EXPOSE 2469
CMD ["./swolemate-server"] 