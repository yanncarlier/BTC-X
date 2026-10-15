# Build stage
FROM rust:1.88-bookworm AS builder

# Set the working directory
WORKDIR /app

# Copy the project files
COPY . /app

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/btcx_api /app/btcx_api

# Expose port 8081
EXPOSE 8081

# Run the binary
CMD ["/app/btcx_api"]