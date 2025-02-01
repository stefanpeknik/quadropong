# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to pre-build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies (this step caches dependencies)
RUN cargo build --release

# Remove the dummy source file
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/server ./pong-server

# Run the application
CMD ["./pong-server"]