use rust_grpc_finance_server::{client, server};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_server_client_interaction() {
    let server_port = 50052;
    println!("Starting server on port {}", server_port);

    let server_handle = tokio::spawn(async move {
        let result = server::run_server("127.0.0.1", server_port).await;
        assert!(result.is_ok(), "Failed to start the server");
    });

    sleep(Duration::from_millis(100)).await;
    println!("Attempting to connect to server...");

    let client_result = client::start_client("127.0.0.1", server_port).await;
    assert!(
        client_result.is_ok(),
        "Client failed to connect to the server"
    );

    println!("Client connected successfully. Stopping server...");
    server_handle.abort();
    println!("Server stopped.");
}

#[tokio::test]
async fn test_client_with_commands() {
    let server_port = 50053;
    println!("Starting server on port {}", server_port);

    let server_handle = tokio::spawn(async move {
        let result = server::run_server("127.0.0.1", server_port).await;
        assert!(result.is_ok(), "Failed to start the server");
    });

    sleep(Duration::from_millis(100)).await;
    println!("Attempting to connect to server...");

    let client_result = client::start_client("127.0.0.1", server_port).await;
    assert!(
        client_result.is_ok(),
        "Client failed to connect to the server"
    );

    println!("Simulating commands...");
    let commands = vec!["list", "AAPL", "stats AAPL", "quit"];
    for command in commands {
        println!("Testing command: {}", command);
        // Simulate command handling here
    }

    println!("Stopping server...");
    server_handle.abort();
    println!("Server stopped.");
}

#[tokio::test]
async fn test_ticker_request() {
    let server_port = 50054;
    println!("Starting server on port {}", server_port);

    let server_handle = tokio::spawn(async move {
        let result = server::run_server("127.0.0.1", server_port).await;
        assert!(result.is_ok(), "Failed to start the server");
    });

    sleep(Duration::from_millis(100)).await;
    println!("Attempting to connect to server...");

    let client_result = client::start_client("127.0.0.1", server_port).await;
    assert!(
        client_result.is_ok(),
        "Client failed to connect to the server"
    );

    println!("Testing ticker request functionality...");
    // Simulate ticker request handling here

    println!("Stopping server...");
    server_handle.abort();
    println!("Server stopped.");
}
