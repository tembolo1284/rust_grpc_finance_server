# Build stage
FROM rust:1.75 AS builder

# Install protobuf compiler
RUN apt-get update && \
    apt-get install -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/rust_grpc_finance_server
COPY . .

# Build the project
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime libraries
RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create config directory
RUN mkdir -p /etc/rust_grpc_finance_server

# Copy the build artifact from the build stage
COPY --from=builder /usr/src/rust_grpc_finance_server/target/release/rust_grpc_finance_server /usr/local/bin/

# Copy the config file
COPY config/config.toml /etc/rust_grpc_finance_server/config.toml

# Create a non-root user
RUN useradd -ms /bin/bash grpcuser && \
    chown -R grpcuser:grpcuser /etc/rust_grpc_finance_server

USER grpcuser

# Set the config file path as an environment variable
ENV CONFIG_PATH=/etc/rust_grpc_finance_server/config.toml

# Command to run the application
ENTRYPOINT ["rust_grpc_finance_server"]
CMD ["server"]
