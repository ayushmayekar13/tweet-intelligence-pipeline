# -------------------------
# 1. Build stage
# -------------------------
    FROM rust:1.82 AS builder

    WORKDIR /app
    
    # Cache dependencies
    COPY Cargo.toml Cargo.lock ./
    RUN mkdir src && echo "fn main() {}" > src/main.rs
    RUN cargo build --release
    RUN rm -rf src
    
    # Copy actual source
    COPY src ./src

    # Force Cargo to realize the file has changed by updating the timestamp
    RUN touch src/main.rs
    # ---------------------
    
    # Build actual binary
    RUN cargo build --release
    
# -------------------------
# 2. Runtime stage
# -------------------------
    FROM debian:bookworm-slim

    # Install OpenSSL libraries and CA certificates
    RUN apt-get update && apt-get install -y \
        libssl3 \
        ca-certificates \
        && rm -rf /var/lib/apt/lists/*
    
    WORKDIR /app
    
    COPY --from=builder /app/target/release/Backend .
    
    EXPOSE 8080
    
    CMD ["./Backend"]