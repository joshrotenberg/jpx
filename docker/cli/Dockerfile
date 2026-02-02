# Dockerfile for jpx CLI
# Used by docker.yml workflow to build images from pre-built release binaries
#
# Usage:
#   echo '{"items": [1,2,3]}' | docker run -i ghcr.io/joshrotenberg/jpx 'length(items)'
#
# The workflow copies the appropriate binary for each platform before building.

FROM alpine:3.21

# Install ca-certificates for HTTPS support and create non-root user
RUN apk add --no-cache ca-certificates \
    && adduser -D -u 1000 jpx

# Copy the pre-built binary (placed in context by workflow)
COPY jpx /usr/local/bin/jpx
RUN chmod +x /usr/local/bin/jpx

# Use non-root user
USER jpx
WORKDIR /home/jpx

# Set entrypoint
ENTRYPOINT ["jpx"]

# Default to showing help
CMD ["--help"]
