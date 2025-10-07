use okx_connector::client::{OKXRestClient, OKXWebSocketClient};
use tokio::sync::mpsc;

/// Configuration for the OKX demo
#[derive(Debug, Clone)]
struct Config {
    /// REST API base URL
    rest_url: String,
    /// WebSocket URL
    ws_url: String,
    /// Trading symbol (e.g., "BTC-USDT", "ETH-USDT")
    symbol: String,
    /// Number of WebSocket updates to display
    update_count: usize,
}

impl Config {
    /// Load configuration from environment variables with defaults
    fn from_env() -> Self {
        Self {
            rest_url: std::env::var("OKX_REST_URL")
                .unwrap_or_else(|_| "https://www.okx.com".to_string()),
            ws_url: std::env::var("OKX_WS_URL")
                .unwrap_or_else(|_| "wss://ws.okx.com:8443/ws/v5/public".to_string()),
            symbol: std::env::var("OKX_SYMBOL").unwrap_or_else(|_| "BTC-USDT".to_string()),
            update_count: std::env::var("OKX_UPDATE_COUNT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        }
    }

    /// Parse configuration from command-line arguments
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut config = Self::from_env();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--rest-url" | "-r" => {
                    if i + 1 < args.len() {
                        config.rest_url = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--ws-url" | "-w" => {
                    if i + 1 < args.len() {
                        config.ws_url = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--symbol" | "-s" => {
                    if i + 1 < args.len() {
                        config.symbol = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--updates" | "-u" => {
                    if i + 1 < args.len() {
                        if let Ok(count) = args[i + 1].parse() {
                            config.update_count = count;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    i += 1;
                }
            }
        }

        config
    }
}

/// Print help message
fn print_help() {
    println!("OKX Connector Demo - Order Book Example");
    println!();
    println!("USAGE:");
    println!("    cargo run --example okx_demo [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -r, --rest-url <URL>     REST API base URL");
    println!("                             [default: https://www.okx.com]");
    println!("                             [env: OKX_REST_URL]");
    println!();
    println!("    -w, --ws-url <URL>       WebSocket URL");
    println!("                             [default: wss://ws.okx.com:8443/ws/v5/public]");
    println!("                             [env: OKX_WS_URL]");
    println!();
    println!("    -s, --symbol <SYMBOL>    Trading symbol");
    println!("                             [default: BTC-USDT]");
    println!("                             [env: OKX_SYMBOL]");
    println!();
    println!("    -u, --updates <COUNT>    Number of WebSocket updates to display");
    println!("                             [default: 10]");
    println!("                             [env: OKX_UPDATE_COUNT]");
    println!();
    println!("    -h, --help               Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Use defaults");
    println!("    cargo run --example okx_demo");
    println!();
    println!("    # Custom symbol");
    println!("    cargo run --example okx_demo --symbol ETH-USDT");
    println!();
    println!("    # Show more updates");
    println!("    cargo run --example okx_demo --updates 20");
    println!();
    println!("    # Using environment variables");
    println!("    OKX_SYMBOL=ETH-USDT cargo run --example okx_demo");
}

/// Format a timestamp in milliseconds to a human-readable date/time
fn format_timestamp(ts: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let duration = UNIX_EPOCH + Duration::from_millis(ts);
    let datetime = chrono::DateTime::<chrono::Utc>::from(duration);
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Print a formatted separator line
fn print_separator() {
    println!("{}", "═".repeat(80));
}

/// Print a section header
fn print_header(title: &str) {
    println!("\n{}", "═".repeat(80));
    println!("  {}", title);
    println!("{}", "═".repeat(80));
}

/// Format a price with proper decimal places and thousands separators
fn format_price(price: f64) -> String {
    let formatted = format!("{:.2}", price);
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts.get(1).unwrap_or(&"00");

    // Add thousands separators
    let mut result = String::new();
    for (i, c) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    let integer_with_commas: String = result.chars().rev().collect();

    format!("{:>12}.{}", integer_with_commas, decimal_part)
}

/// Format an amount with proper decimal places
fn format_amount(amount: f64) -> String {
    format!("{:>15.8}", amount)
}

/// Calculate the total value of an order
fn format_total(price: f64, amount: f64) -> String {
    format_price(price * amount)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_args();

    print_header(&format!(
        "🚀 OKX Connector Demo - {} Order Book",
        config.symbol
    ));

    // Display configuration
    println!("\n⚙️  Configuration:");
    println!("   REST URL:  {}", config.rest_url);
    println!("   WS URL:    {}", config.ws_url);
    println!("   Symbol:    {}", config.symbol);
    println!("   Updates:   {}", config.update_count);

    // Initialize REST client
    println!("\n📡 Connecting to OKX REST API...");
    let rest_client = OKXRestClient::new(&config.rest_url)?;
    println!("✅ Connected successfully!");

    // Fetch order book snapshot
    print_header("📊 Fetching Initial Order Book Snapshot");
    match rest_client.get_order_book(&config.symbol).await {
        Ok(snapshot) => {
            println!("\n⏰ Snapshot Time: {}", format_timestamp(snapshot.ts));
            println!("📈 Total Asks: {}", snapshot.asks.len());
            println!("📉 Total Bids: {}", snapshot.bids.len());

            // Display top asks
            println!(
                "\n┌─ TOP 10 ASKS (Sell Orders) ─────────────────────────────────────────────┐"
            );
            println!(
                "│ {:>3} │ {:>12} │ {:>15} │ {:>12} │",
                "#", "Price (USDT)", "Amount (BTC)", "Total (USDT)"
            );
            println!("├─────┼──────────────┼─────────────────┼──────────────┤");
            for (i, (price, amount)) in snapshot.asks.iter().take(10).enumerate() {
                println!(
                    "│ {:>3} │ {} │ {} │ {} │",
                    i + 1,
                    format_price(*price),
                    format_amount(*amount),
                    format_total(*price, *amount)
                );
            }
            println!(
                "└──────────────────────────────────────────────────────────────────────────┘"
            );

            // Calculate spread
            if let (Some((best_ask, _)), Some((best_bid, _))) =
                (snapshot.asks.first(), snapshot.bids.first())
            {
                let spread = best_ask - best_bid;
                let spread_pct = (spread / best_bid) * 100.0;
                println!("\n💰 Market Spread:");
                println!("   Best Ask:  {}", format_price(*best_ask));
                println!("   Best Bid:  {}", format_price(*best_bid));
                println!(
                    "   Spread:    {} ({:.4}%)",
                    format_price(spread),
                    spread_pct
                );
            }

            // Display top bids
            println!(
                "\n┌─ TOP 10 BIDS (Buy Orders) ──────────────────────────────────────────────┐"
            );
            println!(
                "│ {:>3} │ {:>12} │ {:>15} │ {:>12} │",
                "#", "Price (USDT)", "Amount (BTC)", "Total (USDT)"
            );
            println!("├─────┼──────────────┼─────────────────┼──────────────┤");
            for (i, (price, amount)) in snapshot.bids.iter().take(10).enumerate() {
                println!(
                    "│ {:>3} │ {} │ {} │ {} │",
                    i + 1,
                    format_price(*price),
                    format_amount(*amount),
                    format_total(*price, *amount)
                );
            }
            println!(
                "└──────────────────────────────────────────────────────────────────────────┘"
            );
        }
        Err(e) => {
            eprintln!("\n❌ Error fetching order book: {}", e);
            return Err(e.into());
        }
    }

    // Initialize WebSocket client
    print_header("🔌 Connecting to WebSocket for Real-time Updates");
    let ws_client = OKXWebSocketClient::new(&config.ws_url);

    // Create a channel for receiving WebSocket messages
    let (tx, mut rx) = mpsc::channel(100);

    println!("\n📡 Establishing WebSocket connection...");

    // Spawn a task to handle WebSocket connection and messages
    let symbol = config.symbol.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_client.subscribe_to_order_book(&symbol, tx).await {
            eprintln!("❌ WebSocket error: {}", e);
        }
    });

    println!("✅ WebSocket connected!");
    println!("📨 Subscribing to {} order book updates...", config.symbol);
    println!(
        "\n💡 Receiving live updates (showing first {})...\n",
        config.update_count
    );

    // Main loop to process incoming messages
    let mut update_count = 0;
    for _ in 0..config.update_count {
        match rx.recv().await {
            Some(message) => {
                // Parse the message to determine its type
                if message.contains("\"event\":\"subscribe\"") {
                    println!("✅ Subscription confirmed!");
                    continue;
                }

                update_count += 1;
                println!(
                    "┌─ Update #{} ─────────────────────────────────────────────────────────────┐",
                    update_count
                );

                // Try to parse and display the update nicely
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                        for item in data {
                            if let Some(action) = json.get("action").and_then(|a| a.as_str()) {
                                println!("│ Action: {}", action);
                            }

                            if let Some(ts) = item.get("ts").and_then(|t| t.as_str()) {
                                if let Ok(timestamp) = ts.parse::<u64>() {
                                    println!("│ Time:   {}", format_timestamp(timestamp));
                                }
                            }

                            // Count asks and bids changes
                            let asks_count = item
                                .get("asks")
                                .and_then(|a| a.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0);
                            let bids_count = item
                                .get("bids")
                                .and_then(|b| b.as_array())
                                .map(|b| b.len())
                                .unwrap_or(0);

                            if asks_count > 0 || bids_count > 0 {
                                println!("│ Changes: {} asks, {} bids", asks_count, bids_count);
                            }

                            // Show first few price levels if available
                            if let Some(asks) = item.get("asks").and_then(|a| a.as_array()) {
                                if !asks.is_empty() {
                                    println!("│ Sample Ask Updates:");
                                    for (idx, ask) in asks.iter().take(3).enumerate() {
                                        if let Some(arr) = ask.as_array() {
                                            if arr.len() >= 2 {
                                                let price = arr[0].as_str().unwrap_or("0");
                                                let amount = arr[1].as_str().unwrap_or("0");
                                                println!(
                                                    "│   {}. Price: {}, Amount: {}",
                                                    idx + 1,
                                                    price,
                                                    amount
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(bids) = item.get("bids").and_then(|b| b.as_array()) {
                                if !bids.is_empty() {
                                    println!("│ Sample Bid Updates:");
                                    for (idx, bid) in bids.iter().take(3).enumerate() {
                                        if let Some(arr) = bid.as_array() {
                                            if arr.len() >= 2 {
                                                let price = arr[0].as_str().unwrap_or("0");
                                                let amount = arr[1].as_str().unwrap_or("0");
                                                println!(
                                                    "│   {}. Price: {}, Amount: {}",
                                                    idx + 1,
                                                    price,
                                                    amount
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                println!("└──────────────────────────────────────────────────────────────────────────┘\n");
            }
            None => {
                println!("⚠️  WebSocket channel closed");
                break;
            }
        }
    }

    // Ensure the WebSocket task is properly closed
    ws_handle.abort();

    print_separator();
    println!("✅ Demo completed successfully!");
    println!("📊 Received {} order book updates", update_count);
    print_separator();

    Ok(())
}
