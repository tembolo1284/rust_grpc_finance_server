# RustGRPCFinanceServer

[![Rust gRPC Service CI/CD](https://github.com/tembolo1284/rust_grpc_finance_server/actions/workflows/rust.yml/badge.svg)](https://github.com/tembolo1284/rust_grpc_finance_server/actions/workflows/rust.yml)

## Overview
**RustGRPCFinanceServer** is a gRPC server written in Rust that simulates real-time stock price streaming. It provides price data for predefined tickers (AAPL, MSFT, GOOG, AMZN, META, NFLX, TSLA, NVDA, AMD, INTC), tracks historical prices, and offers statistical analysis capabilities.

## Features
- **gRPC-based Communication**: High-performance, bidirectional streaming between client and server
- **Protocol Buffers**: Strongly typed message definitions for all service interactions
- **Async/Await**: Modern asynchronous programming using Tokio runtime
- **Multiple Price Requests**: Get single or multiple prices for any ticker
- **Historical Data Tracking**: Tracks all prices for each ticker
- **Statistical Analysis**: Provides average and standard deviation for requested tickers
- **Configurable Server**: Reads host and port information from `config/config.toml`
- **Dockerized Deployment**: Complete Docker support with compose and networking
- **CI/CD Pipeline**: GitHub Actions workflow for testing and deployment

## Folder Structure
```plaintext
RustGRPCFinanceServer/

├── proto/

│   └── finance.proto        # Protocol Buffers service definitions

├── src/

│   ├── main.rs             # Main entry point

│   ├── server.rs           # gRPC server implementation

│   ├── client.rs           # gRPC client implementation

│   ├── config.rs           # Configuration file parser

│   ├── utils.rs            # Utility functions

│   └── lib.rs              # Library interface

├── config/

│   └── config.toml         # Configuration file

├── tests/

│   └── integration_tests.rs # Integration tests

├── build.rs                # Protocol Buffers compilation script

├── Dockerfile             # Docker configuration

├── docker-compose.yml     # Docker Compose configuration

├── docker-run.sh         # Docker management script

├── .dockerignore         # Docker ignore file

└── .github/

    └── workflows/

        └── rust.yml      # GitHub Actions CI/CD configuration
```

## Prerequisites
- Rust (install via [rustup](https://rustup.rs/))
- Protocol Buffers Compiler (`protoc`)
  ```bash
  # Ubuntu/Debian
  sudo apt-get install protobuf-compiler
  # macOS
  brew install protobuf
  ```
- Docker (optional, for containerized deployment)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/tembolo1284/rust_grpc_finance_server.git
cd rust_grpc_finance_server
```

2. Build the project:
```bash
cargo build --release
```

## Running the Application

### Local Execution

Start the server:
```bash
cargo run --release -- server
```

Start the client:
```bash
cargo run --release -- client
```

### Docker Execution

Using docker-run.sh (recommended):
```bash
# Build
./docker-run.sh build

# Start server
./docker-run.sh server

# In another terminal, start client
./docker-run.sh client

# Clean up
./docker-run.sh clean
```

Manual Docker commands:
```bash
# Build image
docker build -t rust-grpc-finance-server .

# Run server
docker run -p 50051:50051 rust-grpc-finance-server server

# Run client
docker run --network host rust-grpc-finance-server client
```

## Available Commands
- `list` - Show available tickers
- `stats <ticker>` - Show statistics for a ticker (e.g., `stats GOOG`)
- `<ticker>` - Get current price (e.g., `GOOG`)
- `<ticker> <count>` - Get multiple prices (e.g., `GOOG 5`)
- `quit` or `exit` - Disconnect from server

## Docker Network Setup

1. Create network:
```bash
docker network create finance-net
```

2. Run server in network:
```bash
docker run -d --name grpc-finance-server --network finance-net -p 50051:50051 rust-grpc-finance-server server
```

3. Run client in network:
```bash
docker run --network finance-net rust-grpc-finance-server client
```

## Configuration

Default configuration (`config/config.toml`):
```toml
[server]
host = "0.0.0.0"    # Listen on all interfaces
port = 50051

[client]
host = "grpc-finance-server"  # Docker service name
port = 50051
```

## Testing

Run all tests:
```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture
```

## Service Definition
The gRPC service (`proto/finance.proto`) provides:
- `GetTickerList`: Returns available tickers
- `GetPrice`: Returns current price for a ticker
- `GetMultiplePrices`: Returns multiple prices for a ticker
- `GetStats`: Returns statistical information
- `StreamPrices`: Streams real-time prices (planned feature)

## CI/CD

The project uses GitHub Actions for:
- Running tests
- Code linting (clippy)
- Format checking
- Docker image building
- Container registry publishing

## Author
Paul Nikholas Lopez - [nik.lopez381@gmail.com](mailto:nik.lopez381@gmail.com)

