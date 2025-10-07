# OKX Connector

A Rust library for interacting with the OKX cryptocurrency exchange API. Provides both REST and WebSocket clients for accessing real-time order book data.

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Features

- 🚀 **Async/Await** - Built on Tokio for high-performance async I/O
- 📊 **REST API Client** - Fetch order book snapshots via HTTP
- 🔌 **WebSocket Client** - Real-time order book updates via WebSocket
- 🛡️ **Type-Safe** - Strongly typed order book data structures
- ⚡ **Error Handling** - Comprehensive error types with `thiserror`
- ✅ **Well-Tested** - Unit and integration tests included

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
okx_connector = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### REST API - Fetch Order Book Snapshot

```rust
use okx_connector::OKXRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create REST client
    let client = OKXRestClient::new("https://www.okx.com")?;

    // Fetch order book for BTC-USDT
    let orderbook = client.get_order_book("BTC-USDT").await?;

    println!("Timestamp: {}", orderbook.ts);
    println!("Top ask: {:?}", orderbook.asks.first());
    println!("Top bid: {:?}", orderbook.bids.first());

    Ok(())
}
```

### WebSocket - Real-time Order Book Updates

```rust
use okx_connector::OKXWebSocketClient;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create WebSocket client
    let ws_client = OKXWebSocketClient::new("wss://ws.okx.com:8443/ws/v5/public");

    // Create channel for receiving messages
    let (tx, mut rx) = mpsc::channel(100);

    // Subscribe to order book updates
    tokio::spawn(async move {
        ws_client.subscribe_to_order_book("BTC-USDT", tx).await
    });

    // Process incoming messages
    while let Some(message) = rx.recv().await {
        println!("Received: {}", message);
    }

    Ok(())
}
```

## API Reference

### `OKXRestClient`

REST API client for fetching order book snapshots.

**Methods:**
- `new(base_url: &str) -> Result<Self, OKXClientError>` - Create a new REST client
- `get_order_book(symbol: &str) -> Result<Orderbook, OKXClientError>` - Fetch order book for a symbol

### `OKXWebSocketClient`

WebSocket client for real-time order book updates.

**Methods:**
- `new(url: &str) -> Self` - Create a new WebSocket client
- `subscribe_to_order_book(symbol: &str, tx: mpsc::Sender<String>) -> Result<(), WebSocketError>` - Subscribe to order book updates

### `Orderbook`

Order book data structure with asks and bids.

**Fields:**
- `asks: Vec<(f64, f64)>` - Ask orders (price, amount), sorted ascending by price
- `bids: Vec<(f64, f64)>` - Bid orders (price, amount), sorted descending by price
- `ts: u64` - Timestamp in milliseconds

**Methods:**
- `from_snapshot(data: &str) -> Result<Self, OrderbookError>` - Parse from JSON snapshot
- `apply_update(update: &str) -> Result<(), OrderbookError>` - Apply incremental update

## Running the Example

The repository includes an example that demonstrates both REST and WebSocket functionality:

```bash
# Run with defaults (BTC-USDT)
cargo run --example okx_demo

# Run with custom symbol
cargo run --example okx_demo -- --symbol ETH-USDT

# Show more updates
cargo run --example okx_demo -- --updates 20

# Using environment variables
OKX_SYMBOL=ETH-USDT cargo run --example okx_demo

# View all options
cargo run --example okx_demo -- --help
```

### Configuration Options

The example can be configured via command-line arguments or environment variables:

| Option | Short | Environment Variable | Default | Description |
|--------|-------|---------------------|---------|-------------|
| `--rest-url` | `-r` | `OKX_REST_URL` | `https://www.okx.com` | REST API base URL |
| `--ws-url` | `-w` | `OKX_WS_URL` | `wss://ws.okx.com:8443/ws/v5/public` | WebSocket URL |
| `--symbol` | `-s` | `OKX_SYMBOL` | `BTC-USDT` | Trading symbol |
| `--updates` | `-u` | `OKX_UPDATE_COUNT` | `10` | Number of updates to display |
| `--help` | `-h` | - | - | Show help message |

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_get_order_book
```

## Development

### Prerequisites

- Rust 1.90 or higher
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Code Quality

```bash
# Run clippy linter
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

## Project Structure

```
okx_connector/
├── examples/
│   └── okx_demo.rs          # Example demonstrating REST and WebSocket
├── src/
│   ├── client/
│   │   ├── mod.rs
│   │   ├── rest_client.rs   # REST API client
│   │   └── websocket_client.rs  # WebSocket client
│   ├── models/
│   │   ├── mod.rs
│   │   └── orderbook.rs     # Orderbook data structure
│   ├── utils/
│   │   ├── mod.rs
│   │   └── helpers.rs       # Utility functions
│   └── lib.rs               # Library root
├── Cargo.toml
└── README.md
```

## API Endpoints

### REST API

- **Base URL:** `https://www.okx.com`
- **Endpoint:** `/api/v5/market/books?instId={symbol}`
- **Example:** `https://www.okx.com/api/v5/market/books?instId=BTC-USDT`

### WebSocket API

- **URL:** `wss://ws.okx.com:8443/ws/v5/public`
- **Channel:** `books`
- **Subscribe Message:**
  ```json
  {
    "op": "subscribe",
    "args": [{
      "channel": "books",
      "instId": "BTC-USDT"
    }]
  }
  ```

## Error Handling

The library uses custom error types for better error handling:

- `OKXClientError` - REST client errors (network, parsing, etc.)
- `WebSocketError` - WebSocket connection errors
- `OrderbookError` - Order book parsing and validation errors

All errors implement `std::error::Error` and can be easily propagated using the `?` operator.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Tokio](https://tokio.rs/) for async runtime
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client
- Uses [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) for WebSocket client
