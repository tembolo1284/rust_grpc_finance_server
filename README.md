# RustGRPCFinanceServer

## Overview
**RustGRPCFinanceServer** is a gRPC server written in Rust. It simulates stock price streaming for predefined tickers (AAPL, MSFT, GOOGL, AMZN, META, NFLX, TSLA, NVDA, AMD, INTC) and allows clients to fetch real-time prices. The server tracks historical prices and provides statistics such as average price and standard deviation upon request.

## Features
- **gRPC-based Communication**: High-performance, bidirectional streaming between client and server
- **Protocol Buffers**: Strongly typed message definitions for all service interactions
- **Async/Await**: Modern asynchronous programming using Tokio runtime
- **Simulated Stock Prices**: Streams random prices for predefined tickers
- **Historical Data Tracking**: Tracks all prices for each ticker
- **Statistics**: Provides average and standard deviation for requested tickers
- **Configurable Server**: Reads host and port information from `config/config.toml`
- **Dockerized Deployment**: Run the server in a containerized environment

## Folder Structure
```plaintext
RustGRPCFinanceServer/

│

├── proto/

│   └── finance.proto        # Protocol Buffers service definitions

│

├── src/

│   ├── main.rs             # Main entry point

│   ├── server.rs           # gRPC server implementation

│   ├── client.rs           # gRPC client implementation

│   ├── config.rs           # Configuration file parser

│   ├── utils.rs            # Utility functions (price tracker and random generator)

│   └── lib.rs              # Library interface

│

├── config/

│   └── config.toml         # Configuration file for server host and port

│

├── tests/

│   └── utils_tests.rs      # Unit tests for utility functions

│

├── build.rs               # Protocol Buffers compilation script

├── Dockerfile            # Docker configuration

├── README.md            # Project documentation

├── Cargo.toml           # Rust package configuration

└── .github/

    └── workflows/

        └── rust.yml     # GitHub Actions CI/CD configuration
```

## Prerequisites
- Rust: Install Rust via [rustup](https://rustup.rs/)
- Protocol Buffers Compiler: Install `protoc` for your platform
- Docker (optional): Install Docker for containerized deployment

## Installation

1. Clone the repository:
```bash
git clone <project_url>
cd rust_grpc_finance_server
```

2. Build the project:
```bash
cargo build --release
```

The executable will be located at `target/release/rust_grpc_finance_server`

## Running the Application

### Starting the Server
```bash
cargo run --release -- server
```

The server will start and listen for connections on the host and port specified in `config/config.toml` (default: `127.0.0.1:50051`)

### Starting the Client
```bash
cargo run --release -- client
```

### Available Commands
- `list` - Get a list of available tickers
- `stats <ticker>` - Get statistics for a specific ticker (e.g., `stats AAPL`)
- `<ticker>` - Get current price for a specific ticker (e.g., `AMZN`)
- `quit` or `exit` - Disconnect from the server

## Docker Deployment

1. Build the Docker image:
```bash
docker build -t rust-grpc-finance-server .
```

2. Run the container (server):
```bash
docker run -p 50051:50051 rust-grpc-finance-server server
```

3. Run a detached container (server):
```bash
docker run -d -p 50051:50051 rust-grpc-finance-server server
```

4. Run the client:
```bash
docker run rust-grpc-finance-server client
```

### Useful Docker Commands
```bash
# View running containers
docker ps

# View container logs
docker logs <container-id>

# Stop container
docker stop <container-id>
```

## Docker Network Setup

1. Stop any running containers:
```bash
docker stop $(docker ps -a -q)
```

2. Remove existing containers:
```bash
docker rm finance-server 2>/dev/null || true
```

3. Create network (if not exists):
```bash
docker network create finance-net
```

4. Start server in network:
```bash
docker run -d --name finance-server --network finance-net -p 50051:50051 rust-grpc-finance-server server
```

5. Run client in network:
```bash
docker run --network finance-net rust-grpc-finance-server client
```

## Testing

Run the test suite:
```bash
cargo test
```

## Configuration

The server configuration is stored in `config/config.toml`:
```toml
[server]
host = "127.0.0.1"
port = 50051

[client]
host = "127.0.0.1"
port = 50051
```

## Service Definition
The gRPC service is defined in `proto/finance.proto` and provides the following methods:
- `GetTickerList`: Returns list of available tickers
- `GetPrice`: Returns current price for a ticker
- `GetStats`: Returns statistical information for a ticker
- `StreamPrices`: Streams real-time prices for a ticker (planned feature)

## Author
Paul Nikholas Lopez - [nik.lopez381@gmail.com](mailto:nik.lopez381@gmail.com)
