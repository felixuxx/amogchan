# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -s /bin/bash appuser

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/encrypted-social-platform /app/
COPY --from=builder /app/static /app/static/
COPY --from=builder /app/migrations /app/migrations/

# Change ownership to app user
RUN chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 3000

# Run the application
CMD ["./encrypted-social-platform"]