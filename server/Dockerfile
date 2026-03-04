# Stage 1: Plan
FROM rust:1.93-slim AS planner
WORKDIR /usr/src/app
ENV SQLX_OFFLINE=true

# Install build dependencies and cargo-chef
RUN apt-get update && \
    apt-get install -y pkg-config libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-chef --locked --target "$(rustc -vV | sed -n 's/^host: //p')"

# Copy source code for planning
COPY . .

# Create the recipe
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build
FROM rust:1.93-slim AS builder
WORKDIR /usr/src/app
ENV SQLX_OFFLINE=true

# Default to amd64 musl; adjust if you build multi-arch.
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libsqlite3-dev musl-tools && \
    rm -rf /var/lib/apt/lists/*

# Copy Cargo Chef from planner (avoids reinstall + target issues)
COPY --from=planner /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef

# Install musl target
RUN rustup target add ${CARGO_BUILD_TARGET}

# Copy the recipe from the planner
COPY --from=planner /usr/src/app/recipe.json recipe.json

# Build dependencies - this is the caching layer
RUN cargo chef cook --release --target ${CARGO_BUILD_TARGET} --recipe-path recipe.json

# Copy source code
COPY . .

# Build the binary
RUN cargo build --release --target ${CARGO_BUILD_TARGET}

# Prepare runtime data dirs for distroless image (needs to exist and be writable).
RUN mkdir -p /usr/src/app/runtime-data/database /usr/src/app/runtime-data/backups

# Stage 3: Run
FROM gcr.io/distroless/static:nonroot
WORKDIR /data
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl

# Copy runtime data dirs and the binary (distroless has no shell to create/chown at runtime).
COPY --from=builder --chown=nonroot:nonroot /usr/src/app/runtime-data/ /data/
COPY --from=builder --chown=nonroot:nonroot /usr/src/app/target/${CARGO_BUILD_TARGET}/release/swolemate-server /app/swolemate-server

EXPOSE 2469
ENTRYPOINT ["/app/swolemate-server"]
