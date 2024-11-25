use tonic::{Request, Response, Status};
use crate::finance::{
    TickerListRequest, TickerListResponse,
    PriceRequest, PriceResponse,
    MultiplePricesRequest, MultiplePricesResponse,
    StatsRequest, StatsResponse,
};
use super::service::StockServiceImpl;

impl StockServiceImpl {
    pub(crate) async fn handle_get_ticker_list(
        &self,
        request: Request<TickerListRequest>,
    ) -> Result<Response<TickerListResponse>, Status> {
        let remote_addr = request
            .remote_addr()
            .unwrap_or_else(|| "unknown".parse().unwrap());
        println!("Received ticker list request from {}", remote_addr);

        let response = TickerListResponse {
            tickers: crate::utils::TICKERS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
        };

        println!("Sending ticker list response: {:?}", response.tickers);
        Ok(Response::new(response))
    }

    pub(crate) async fn handle_get_price(
        &self,
        request: Request<PriceRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let remote_addr = request
            .remote_addr()
            .unwrap_or_else(|| "unknown".parse().unwrap());
        let ticker = request.into_inner().ticker.to_uppercase();
        println!(
            "Received price request for ticker: {} from {}",
            ticker, remote_addr
        );

        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            println!("Error: Invalid ticker requested: {}", ticker);
            return Err(Status::invalid_argument(format!(
                "Invalid ticker: {}",
                ticker
            )));
        }

        let (_, price) = crate::utils::generate_random_ticker_and_price();
        let formatted_message = crate::utils::format_price(&ticker, price);

        let mut tracker = self.price_tracker.lock().await;
        tracker.add_price(&ticker, price);

        println!("Sending price response: {}", formatted_message.trim());
        Ok(Response::new(PriceResponse {
            ticker,
            price,
            formatted_message,
        }))
    }

    pub(crate) async fn handle_get_multiple_prices(
        &self,
        request: Request<MultiplePricesRequest>,
    ) -> Result<Response<MultiplePricesResponse>, Status> {
        let remote_addr = request
            .remote_addr()
            .unwrap_or_else(|| "unknown".parse().unwrap());
        let req = request.into_inner();
        let ticker = req.ticker.to_uppercase();
        let count = req.count;

        println!(
            "Received multiple prices request for ticker: {} (count: {}) from {}",
            ticker, count, remote_addr
        );

        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            return Err(Status::invalid_argument(format!(
                "Invalid ticker: {}",
                ticker
            )));
        }

        let mut prices = Vec::with_capacity(count as usize);
        let mut tracker = self.price_tracker.lock().await;

        for _ in 0..count {
            let (_, price) = crate::utils::generate_random_ticker_and_price();
            tracker.add_price(&ticker, price);
            prices.push(price);
        }

        let formatted_message = format!("Generated {} prices for {}", count, ticker);
        println!("Sending multiple prices response: {}", formatted_message);

        Ok(Response::new(MultiplePricesResponse {
            ticker,
            prices,
            formatted_message,
        }))
    }

    pub(crate) async fn handle_get_stats(
        &self,
        request: Request<StatsRequest>,
    ) -> Result<Response<StatsResponse>, Status> {
        let remote_addr = request
            .remote_addr()
            .unwrap_or_else(|| "unknown".parse().unwrap());
        let ticker = request.into_inner().ticker.to_uppercase();
        println!(
            "Received stats request for ticker: {} from {}",
            ticker, remote_addr
        );

        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            return Err(Status::invalid_argument(format!(
                "Invalid ticker: {}",
                ticker
            )));
        }

        let tracker = self.price_tracker.lock().await;
        let (prices, average, std_deviation) = tracker.get_stats(&ticker);

        let formatted_message = format!(
            "{} Statistics:\nAverage: ${:.2}\nStd Dev: ${:.2}\nSample Size: {}",
            ticker,
            average,
            std_deviation,
            prices.len()
        );

        println!("Sending stats response for ticker: {}", ticker);
        Ok(Response::new(StatsResponse {
            ticker,
            prices,
            average,
            std_deviation,
            formatted_message,
        }))
    }
}
