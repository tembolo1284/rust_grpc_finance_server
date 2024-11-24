// Update the main loop in your client.rs where it handles commands:

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
            ticker_input => {
                // Split the input to check for count
                let parts: Vec<&str> = ticker_input.split_whitespace().collect();
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
