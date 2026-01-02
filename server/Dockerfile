# Stage 1: Plan
FROM rust:1.92-slim as planner
WORKDIR /usr/src/app

# Install build dependencies and cargo-chef
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-chef

# Copy source code for planning
COPY . .

# Create the recipe
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build
FROM rust:1.92-slim as builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

# Install Cargo Chef
RUN cargo install cargo-chef

# Copy the recipe from the planner
COPY --from=planner /usr/src/app/recipe.json recipe.json

# Build dependencies - this is the caching layer
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Build the binary
RUN cargo build --release

# Stage 3: Run
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

# Copy only the binary
COPY --from=builder /usr/src/app/target/release/swolemate-server ./

EXPOSE 2469
CMD ["./swolemate-server"] 
