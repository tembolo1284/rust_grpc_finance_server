use rust_grpc_finance_server::finance::{
    stock_service_client::StockServiceClient,
    TickerListRequest, PriceRequest, StatsRequest,
    MultiplePricesRequest,
};
use rust_grpc_finance_server::server::StockServiceImpl;
use tokio;
use std::time::Duration;

async fn spawn_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        StockServiceImpl::run_server("127.0.0.1", port).await.unwrap();
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

    // Test getting multiple prices for a ticker
    let count = 5;
    let response = client.get_multiple_prices(MultiplePricesRequest {
        ticker: ticker.clone(),
        count,
    }).await?;
    let multiple_prices = response.into_inner();
    assert_eq!(&multiple_prices.ticker, ticker);
    assert_eq!(multiple_prices.prices.len(), count as usize);
    for price in multiple_prices.prices {
        assert!(price > 0.0);
    }

    // Test getting stats
    let response = client.get_stats(StatsRequest {
        ticker: ticker.clone(),
    }).await?;
    let stats = response.into_inner();
    assert_eq!(&stats.ticker, ticker);
    assert!(!stats.prices.is_empty());
    assert!(stats.average > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_invalid_ticker() -> Result<(), Box<dyn std::error::Error>> {
    // Start server in background
    let _server_handle = spawn_test_server(50052).await;

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = StockServiceClient::connect("http://127.0.0.1:50052").await?;

    // Test invalid ticker handling for single price
    let response = client.get_price(PriceRequest {
        ticker: "INVALID".to_string(),
    }).await;
    assert!(response.is_err());
    assert!(response.unwrap_err().message().contains("Invalid ticker"));

    // Test invalid ticker handling for multiple prices
    let response = client.get_multiple_prices(MultiplePricesRequest {
        ticker: "INVALID".to_string(),
        count: 5,
    }).await;
    assert!(response.is_err());
    assert!(response.unwrap_err().message().contains("Invalid ticker"));

    // Test invalid ticker handling for stats
    let response = client.get_stats(StatsRequest {
        ticker: "INVALID".to_string(),
    }).await;
    assert!(response.is_err());
    assert!(response.unwrap_err().message().contains("Invalid ticker"));
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_prices_invalid_count() -> Result<(), Box<dyn std::error::Error>> {
    // Start server in background
    let _server_handle = spawn_test_server(50053).await;

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = StockServiceClient::connect("http://127.0.0.1:50053").await?;

    // Test negative count
    let response = client.get_multiple_prices(MultiplePricesRequest {
        ticker: "AAPL".to_string(),
        count: -1,
    }).await;
    assert!(response.is_err());
    assert!(response.unwrap_err().message().contains("Count must be positive"));

    // Test zero count
    let response = client.get_multiple_prices(MultiplePricesRequest {
        ticker: "AAPL".to_string(),
        count: 0,
    }).await;
    assert!(response.is_err());
    assert!(response.unwrap_err().message().contains("Count must be positive"));

    Ok(())
}
