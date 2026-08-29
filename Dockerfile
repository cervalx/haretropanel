# 1. Build stage
FROM rust:1.83 AS builder

WORKDIR /app

# Cache-friendly dependency build: copy manifest first
COPY Cargo.toml Cargo.lock* ./
COPY templates ./templates
COPY src ./src

# Release build
RUN cargo build --release

# 2. Runtime stage - minimal Linux
FROM debian:bookworm-slim

# Add CA certificates for HTTPS (HA, etc.)
RUN rm -rf /var/lib/apt/lists/* \
	&& apt-get update \
	&& DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and templates
COPY --from=builder /app/target/release/haretropanel /app/haretropanel
COPY templates ./templates

# Default env
ENV HARETROPANEL_PORT=8080

EXPOSE 8080

CMD ["./haretropanel"]