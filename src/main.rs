use std::error::Error;
use rust_tcp_finance_server::{config, server, client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = config::load_config().expect("Failed to load configuration");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    match args.get(1).map(String::as_str) {
        Some("server") => {
            println!("Starting server...");
            server::run_server(&config.server.host, config.server.port).await?;
        },
        Some("client") => {
            println!("Starting client...");
            client::start_client(&config.client.host, config.client.port).await?;
        },
        _ => {
            println!("Usage:");
            println!("  {} server   - Start the server", args[0]);
            println!("  {} client   - Start the client", args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = config::load_config();
        assert!(config.is_ok(), "Configuration should load successfully");
    }
}
