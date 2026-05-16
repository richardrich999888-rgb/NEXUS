FROM rust:1.91-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build release binary with all production features
RUN cargo build --release --features production

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/causalux-v2 /usr/local/bin/causalux

# Create data directory
RUN mkdir -p /data

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Run as non-root user
RUN useradd -m -u 1000 causalux && \
    chown -R causalux:causalux /app /data
USER causalux

# Set environment
ENV RUST_LOG=info
ENV DATA_DIR=/data

ENTRYPOINT ["causalux"]
CMD ["--port", "8080"]
