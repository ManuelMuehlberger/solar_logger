# Build stage
FROM rust:latest as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/app

# Set build arguments for Git credentials
ARG GIT_USERNAME
ARG GIT_PASSWORD

# Clone repository using inline credentials and change to backend directory
RUN git clone https://${GIT_USERNAME}:${GIT_PASSWORD}@mygit.7oo.xyz/manu/solarmeter.git . && \
    cd backend

# Build the application from the backend directory
WORKDIR /usr/src/app/backend
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create necessary directories
RUN mkdir -p /var/log/solar_meter /app/db

# Copy the built binary from the backend directory
COPY --from=builder /usr/src/app/backend/target/release/solarmeter /usr/local/bin/

# Copy configuration from the backend directory
COPY --from=builder /usr/src/app/backend/src/config.toml /app/config/config.toml

# Create a non-root user
RUN useradd -r -s /bin/false solarmeter && \
    chown -R solarmeter:solarmeter /var/log/solar_meter /app

# Switch to non-root user
USER solarmeter

# Set default configuration path
ENV CONFIG_PATH=/app/config/config.toml

# Expose the metrics port
EXPOSE 8081

# Command to run the application
CMD ["solarmeter"]