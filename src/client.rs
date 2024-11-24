use tokio::io::{AsyncBufReadExt, BufReader};
use crate::finance::stock_service_client::StockServiceClient;
use crate::finance::{
    TickerListRequest, PriceRequest, StatsRequest,
    MultiplePricesRequest,
};

pub async fn start_client(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Add retry logic for Docker container startup timing
    let max_retries = 5;
    let mut retry_count = 0;
    let addr = format!("http://{}:{}", host, port);
    
    println!("Attempting to connect to {}", addr);

    let mut client = loop {
        match StockServiceClient::connect(addr.clone()).await {
            Ok(client) => break client,
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(format!("Failed to connect after {} attempts: {}", max_retries, e).into());
                }
                println!("Connection attempt {} failed, retrying in 2 seconds...", retry_count);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    };

    println!("Successfully connected to gRPC server");
    println!("\nAvailable commands:");
    println!("- list: Show available tickers");
    println!("- stats <ticker>: Show statistics for a ticker");
    println!("- <ticker> [count]: Get current price(s) for a ticker");
    println!("- quit or exit: Disconnect from server\n");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();

    loop {
        println!("Enter a command:");
        buffer.clear();
        
        if reader.read_line(&mut buffer).await? == 0 {
            break;
        }

        let command = buffer.trim();
        if command.is_empty() {
            continue;
        }

        if command == "list" {
            match client.get_ticker_list(TickerListRequest {}).await {
                Ok(response) => {
                    let tickers = response.into_inner().tickers;
                    println!("Available tickers: {}", tickers.join(", "));
                },
                Err(e) => eprintln!("Error getting ticker list: {}", e),
            }
        } else if command == "quit" || command == "exit" {
            println!("Disconnecting from server...");
            break;
        } else if command.starts_with("stats ") {
            let ticker = command.strip_prefix("stats ").unwrap().to_string();
            match client.get_stats(StatsRequest { ticker }).await {
                Ok(response) => {
                    let stats = response.into_inner();
                    println!("{}", stats.formatted_message);
                },
                Err(e) => eprintln!("Error getting stats: {}", e),
            }
        } else {
            // Handle ticker requests (single price or multiple prices)
            let parts: Vec<&str> = command.split_whitespace().collect();
            match parts.as_slice() {
                [ticker, count_str] => {
                    // Try to parse the count
                    match count_str.parse::<i32>() {
                        Ok(count) => {
                            match client.get_multiple_prices(MultiplePricesRequest {
                                ticker: ticker.to_string(),
                                count,
                            }).await {
                                Ok(response) => {
                                    println!("{}", response.into_inner().formatted_message);
                                },
                                Err(e) => eprintln!("Error getting multiple prices: {}", e),
                            }
                        },
                        Err(_) => eprintln!("Invalid number format for count"),
                    }
                },
                [ticker] => {
                    // Single price request
                    match client.get_price(PriceRequest { 
                        ticker: ticker.to_string() 
                    }).await {
                        Ok(response) => {
                            let price_info = response.into_inner();
                            println!("{}", price_info.formatted_message);
                        },
                        Err(e) => eprintln!("Error getting price: {}", e),
                    }
                },
                _ => println!("Invalid command format"),
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_client_connection() {
        // This is a basic test to ensure the client connection can be created
        let result = StockServiceClient::connect("http://[::1]:50051").await;
        assert!(result.is_err()); // Should fail as no server is running
    }
}
