# ---- BUILDER STAGE ----
# Use a Rust image with NASM installed for building
FROM rust:latest AS builder

# Install NASM and other potential build dependencies
RUN apt-get update && \
    apt-get install -y nasm && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest and lock files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the project in release mode
RUN cargo build --release

# ---- RUNTIME STAGE ----
# Use a minimal Debian image for final runtime
FROM debian:stable-slim

# Install minimal runtime dependencies (adjust based on your app needs)
RUN apt-get update && \
    apt-get install -y libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/muse /app/muse

# Copy the runtime directory and .env file
COPY runtime /app/runtime
COPY .env /app/.env

# Expose the port if your app listens on one (adjust accordingly)
EXPOSE 8000

# Command to run your app
CMD ["./muse"]