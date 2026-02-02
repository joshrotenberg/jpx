# Dockerfile for jpx-server MCP server
# Builds from source for all platforms
#
# Usage with Claude Desktop or other MCP clients:
#   docker run -i ghcr.io/joshrotenberg/jpx-server
#
# The server communicates via stdin/stdout for MCP transport.

# Build stage
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo build --release -p jpx-server && \
    strip target/release/jpx-server

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates \
    && adduser -D -u 1000 jpx

COPY --from=builder /build/target/release/jpx-server /usr/local/bin/jpx-server
RUN chmod +x /usr/local/bin/jpx-server

USER jpx
WORKDIR /home/jpx

ENTRYPOINT ["jpx-server"]
