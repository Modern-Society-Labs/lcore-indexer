# Build stage
FROM rust:1.82 AS builder

WORKDIR /app

# Install nightly toolchain for edition2024 support
RUN rustup toolchain install nightly
RUN rustup default nightly

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build release binary with nightly
RUN cargo +nightly build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/lcore-indexer /usr/local/bin/

# Copy migrations
COPY migrations ./migrations

# Create non-root user
RUN useradd -m -u 1001 indexer && \
    chown -R indexer:indexer /app

# Set environment variables for Railway
ENV INDEXER_API_HOST="0.0.0.0"
ENV INDEXER_API_PORT="8090"
ENV INDEXER_START_BLOCK="153950"

USER indexer

# Expose API port
EXPOSE 8090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8090/health || exit 1

# Run the indexer
CMD ["lcore-indexer"]
