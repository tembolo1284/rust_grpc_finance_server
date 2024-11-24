use rust_grpc_finance_server::finance::{
    stock_service_client::StockServiceClient,
    TickerListRequest, PriceRequest, StatsRequest
};
use rust_grpc_finance_server::server;
use tokio;
use std::time::Duration;

async fn spawn_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        server::run_server("127.0.0.1", port).await.unwrap();
    })
}

#[tokio::test]
async fn test_full_client_server_interaction() -> Result<(), Box<dyn std::error::Error>> {
    // Start server in background
    let _server_handle = spawn_test_server(50051).await;

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = StockServiceClient::connect("http://127.0.0.1:50051").await?;

    // Test getting ticker list
    let response = client.get_ticker_list(TickerListRequest {}).await?;
    let tickers = response.into_inner().tickers;
    assert!(!tickers.is_empty());

    // Test getting price for a ticker
    let ticker = &tickers[0]; // Use first available ticker
    let response = client.get_price(PriceRequest {
        ticker: ticker.clone(),
    }).await?;
    let price_info = response.into_inner();
    assert_eq!(&price_info.ticker, ticker);
    assert!(price_info.price > 0.0);

    // Test getting stats
    let response = client.get_stats(StatsRequest {
        ticker: ticker.clone(),
    }).await?;
    let stats = response.into_inner();
    assert_eq!(&stats.ticker, ticker);

    Ok(())
}

#[tokio::test]
async fn test_invalid_ticker() -> Result<(), Box<dyn std::error::Error>> {
    // Start server in background
    let _server_handle = spawn_test_server(50052).await;

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = StockServiceClient::connect("http://127.0.0.1:50052").await?;

    // Test invalid ticker handling
    let response = client.get_price(PriceRequest {
        ticker: "INVALID".to_string(),
    }).await;
    
    assert!(response.is_err());
    assert!(response.unwrap_err().message().contains("Invalid ticker"));
    
    Ok(())
}
