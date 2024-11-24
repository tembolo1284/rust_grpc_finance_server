use tokio::io::{AsyncBufReadExt, BufReader};
use crate::finance::stock_service_client::StockServiceClient;
use crate::finance::{TickerListRequest, PriceRequest, StatsRequest};

pub async fn start_client(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", host, port);
    let mut client = StockServiceClient::connect(addr).await?;

    println!("Connected to gRPC server");
    println!("\nAvailable commands:");
    println!("- list: Show available tickers");
    println!("- stats <ticker>: Show statistics for a ticker");
    println!("- <ticker>: Get current price for a ticker");
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

        match command {
            "list" => {
                match client.get_ticker_list(TickerListRequest {}).await {
                    Ok(response) => {
                        let tickers = response.into_inner().tickers;
                        println!("Available tickers: {}", tickers.join(", "));
                    },
                    Err(e) => eprintln!("Error getting ticker list: {}", e),
                }
            },
            "quit" | "exit" => {
                println!("Disconnecting from server...");
                break;
            },
            cmd if cmd.starts_with("stats ") => {
                let ticker = cmd.strip_prefix("stats ").unwrap().to_string();
                match client.get_stats(StatsRequest { ticker }).await {
                    Ok(response) => {
                        let stats = response.into_inner();
                        println!("{}", stats.formatted_message);
                    },
                    Err(e) => eprintln!("Error getting stats: {}", e),
                }
            },
            ticker => {
                match client.get_price(PriceRequest { 
                    ticker: ticker.to_string() 
                }).await {
                    Ok(response) => {
                        let price_info = response.into_inner();
                        println!("{}", price_info.formatted_message);
                    },
                    Err(e) => eprintln!("Error getting price: {}", e),
                }
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
        // This is a basic test to ensure the client can be created
        // In a real test, you'd want to mock the server
        let result = StockServiceClient::connect("http://[::1]:50051").await;
        assert!(result.is_err()); // Should fail as no server is running
    }
}
