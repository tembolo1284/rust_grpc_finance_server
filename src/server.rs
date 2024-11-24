// Add this to your existing server.rs, inside the StockService implementation block:

    async fn get_multiple_prices(
        &self,
        request: Request<MultiplePricesRequest>,
    ) -> Result<Response<MultiplePricesResponse>, Status> {
        let remote_addr = request.remote_addr();
        let req = request.into_inner();
        let ticker = req.ticker.to_uppercase();
        let count = req.count;
        
        println!("Received multiple prices request for ticker: {} (count: {}) from {:?}", 
                ticker, count, remote_addr);
        
        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            println!("Error: Invalid ticker requested: {}", ticker);
            return Err(Status::invalid_argument(format!("Invalid ticker: {}", ticker)));
        }

        if count <= 0 {
            return Err(Status::invalid_argument("Count must be positive"));
        }

        let mut prices = Vec::with_capacity(count as usize);
        let mut formatted_messages = Vec::new();

        for _ in 0..count {
            let (_, price) = crate::utils::generate_random_ticker_and_price();
            prices.push(price);
            formatted_messages.push(format!("${:.2}", price));
        }

        let formatted_message = format!(
            "Prices for {}:\n{}", 
            ticker,
            formatted_messages.join("\n")
        );

        {
            let mut tracker = self.price_tracker.lock().await;
            for &price in &prices {
                tracker.add_price(&ticker, price);
            }
        }

        println!("Sending multiple prices response:\n{}", formatted_message);
        Ok(Response::new(MultiplePricesResponse {
            ticker,
            prices,
            formatted_message,
        }))
    }
