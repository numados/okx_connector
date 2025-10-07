use clap::Parser;
use okx_connector::client::{OKXRestClient, OKXWebSocketClient};
use tokio::sync::mpsc;

/// OKX Connector Demo - Order Book Example
///
/// This example demonstrates how to use the OKX connector library to:
/// - Fetch order book snapshots via REST API
/// - Subscribe to real-time order book updates via WebSocket
#[derive(Parser, Debug)]
#[command(name = "okx_demo")]
#[command(author, version, about, long_about = None)]
struct Config {
    /// REST API base URL
    #[arg(
        short = 'r',
        long,
        env = "OKX_REST_URL",
        default_value = "https://www.okx.com"
    )]
    rest_url: String,

    /// WebSocket URL
    #[arg(
        short = 'w',
        long,
        env = "OKX_WS_URL",
        default_value = "wss://ws.okx.com:8443/ws/v5/public"
    )]
    ws_url: String,

    /// Trading symbol (e.g., "BTC-USDT", "ETH-USDT", "SOL-USDT")
    #[arg(short = 's', long, env = "OKX_SYMBOL", default_value = "BTC-USDT")]
    symbol: String,

    /// Number of WebSocket updates to display
    #[arg(short = 'u', long, env = "OKX_UPDATE_COUNT", default_value = "10")]
    update_count: usize,
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
    // Parse configuration from command-line arguments and environment variables
    let config = Config::parse();

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
