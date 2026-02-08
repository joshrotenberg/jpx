# Dockerfile for jpx-mcp MCP server
# Builds from source for all platforms
#
# Usage with Claude Desktop or other MCP clients:
#   docker run -i ghcr.io/joshrotenberg/jpx-mcp
#
# The server communicates via stdin/stdout for MCP transport.

# Build stage
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo build --release -p jpx-mcp && \
    strip target/release/jpx-mcp

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates \
    && adduser -D -u 1000 jpx

COPY --from=builder /build/target/release/jpx-mcp /usr/local/bin/jpx-mcp
RUN chmod +x /usr/local/bin/jpx-mcp

USER jpx
WORKDIR /home/jpx

ENTRYPOINT ["jpx-mcp"]
CMD ["--help"]
