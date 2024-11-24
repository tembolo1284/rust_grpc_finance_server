use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::finance::stock_service_server::{StockService, StockServiceServer};
use crate::finance::{
    MultiplePricesRequest, MultiplePricesResponse, PriceRequest, PriceResponse, StatsRequest,
    StatsResponse, TickerListRequest, TickerListResponse,
};
use crate::utils::PriceTracker;

pub struct StockServiceImpl {
    price_tracker: Arc<Mutex<PriceTracker>>,
}

impl StockServiceImpl {
    fn new() -> Self {
        StockServiceImpl {
            price_tracker: Arc::new(Mutex::new(PriceTracker::new())),
        }
    }

    pub async fn run_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", host, port).parse()?;
        let service = Self::new();

        println!("Server starting up...");
        println!("Server listening on {}", addr);

        Server::builder()
            .add_service(StockServiceServer::new(service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl StockService for StockServiceImpl {
    async fn get_ticker_list(
        &self,
        request: Request<TickerListRequest>,
    ) -> Result<Response<TickerListResponse>, Status> {
        println!(
            "Received ticker list request from {:?}",
            request.remote_addr()
        );

        let response = TickerListResponse {
            tickers: crate::utils::TICKERS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
        };

        println!("Sending ticker list response: {:?}", response.tickers);
        Ok(Response::new(response))
    }

    async fn get_price(
        &self,
        request: Request<PriceRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let remote_addr = request.remote_addr();
        let ticker = request.into_inner().ticker.to_uppercase();
        println!(
            "Received price request for ticker: {} from {:?}",
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

    async fn get_multiple_prices(
        &self,
        request: Request<MultiplePricesRequest>,
    ) -> Result<Response<MultiplePricesResponse>, Status> {
        let remote_addr = request.remote_addr();
        let req = request.into_inner();
        let ticker = req.ticker.to_uppercase();
        let count = req.count;

        println!(
            "Received multiple prices request for ticker: {} (count: {}) from {:?}",
            ticker, count, remote_addr
        );

        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            println!("Error: Invalid ticker requested: {}", ticker);
            return Err(Status::invalid_argument(format!(
                "Invalid ticker: {}",
                ticker
            )));
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

        let formatted_message =
            format!("Prices for {}:\n{}", ticker, formatted_messages.join("\n"));

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

    async fn get_stats(
        &self,
        request: Request<StatsRequest>,
    ) -> Result<Response<StatsResponse>, Status> {
        let remote_addr = request.remote_addr();
        let ticker = request.into_inner().ticker.to_uppercase();
        println!(
            "Received stats request for ticker: {} from {:?}",
            ticker, remote_addr
        );

        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            println!("Error: Invalid ticker requested: {}", ticker);
            return Err(Status::invalid_argument(format!(
                "Invalid ticker: {}",
                ticker
            )));
        }

        let tracker = self.price_tracker.lock().await;

        let prices = match tracker.get_prices(&ticker) {
            Some(p) => p.clone(),
            None => {
                println!("Error: No data available for ticker: {}", ticker);
                return Err(Status::not_found(format!(
                    "No data available for ticker: {}",
                    ticker
                )));
            }
        };

        let average = tracker.average(&ticker).unwrap_or(0.0);
        let std_deviation = tracker.std_deviation(&ticker).unwrap_or(0.0);

        let formatted_message = format!(
            "Stats for {}:\nPrices: {:?}\nAverage: {:.2}\nStd Dev: {:.2}\n",
            ticker, prices, average, std_deviation
        );

        println!("Sending stats response:\n{}", formatted_message.trim());
        Ok(Response::new(StatsResponse {
            ticker,
            prices,
            average,
            std_deviation,
            formatted_message,
        }))
    }

    type StreamPricesStream =
        Pin<Box<dyn Stream<Item = Result<PriceResponse, Status>> + Send + 'static>>;

    async fn stream_prices(
        &self,
        request: Request<PriceRequest>,
    ) -> Result<Response<Self::StreamPricesStream>, Status> {
        let remote_addr = request.remote_addr();
        let ticker = request.into_inner().ticker.to_uppercase();
        println!(
            "Received streaming request for ticker: {} from {:?}",
            ticker, remote_addr
        );

        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            println!("Error: Invalid ticker requested: {}", ticker);
            return Err(Status::invalid_argument(format!(
                "Invalid ticker: {}",
                ticker
            )));
        }

        let (tx, rx) = mpsc::channel(32);
        let price_tracker = self.price_tracker.clone();
        let stream_ticker = ticker.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            println!("Starting price stream for ticker: {}", ticker);

            loop {
                interval.tick().await;
                let (_, price) = crate::utils::generate_random_ticker_and_price();
                let formatted_message = crate::utils::format_price(&ticker, price);

                {
                    let mut tracker = price_tracker.lock().await;
                    tracker.add_price(&ticker, price);
                }

                println!("Streaming price: {}", formatted_message.trim());

                if tx
                    .send(Ok(PriceResponse {
                        ticker: ticker.clone(),
                        price,
                        formatted_message,
                    }))
                    .await
                    .is_err()
                {
                    println!(
                        "Client disconnected from price stream for ticker: {}",
                        ticker
                    );
                    break;
                }
            }
        });

        println!("Established price stream for ticker: {}", stream_ticker);
        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::StreamPricesStream
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_ticker_list() {
        let service = StockServiceImpl::new();
        let request = Request::new(TickerListRequest {});
        let response = service.get_ticker_list(request).await.unwrap();
        assert!(!response.into_inner().tickers.is_empty());
    }

    #[tokio::test]
    async fn test_get_price() {
        let service = StockServiceImpl::new();
        let request = Request::new(PriceRequest {
            ticker: "AAPL".to_string(),
        });
        let response = service.get_price(request).await.unwrap();
        let price_response = response.into_inner();
        assert_eq!(price_response.ticker, "AAPL");
        assert!(price_response.price > 0.0);
    }

    #[tokio::test]
    async fn test_get_price_invalid_ticker() {
        let service = StockServiceImpl::new();
        let request = Request::new(PriceRequest {
            ticker: "INVALID".to_string(),
        });
        let result = service.get_price(request).await;
        assert!(result.is_err());
    }
}
