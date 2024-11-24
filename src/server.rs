use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use futures::Stream;
use std::pin::Pin;

use crate::finance::stock_service_server::{StockService, StockServiceServer};
use crate::finance::{
    TickerListRequest, TickerListResponse,
    PriceRequest, PriceResponse,
    StatsRequest, StatsResponse,
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
}

#[tonic::async_trait]
impl StockService for StockServiceImpl {
    async fn get_ticker_list(
        &self,
        _request: Request<TickerListRequest>,
    ) -> Result<Response<TickerListResponse>, Status> {
        Ok(Response::new(TickerListResponse {
            tickers: crate::utils::TICKERS.iter().map(|&s| s.to_string()).collect(),
        }))
    }

    async fn get_price(
        &self,
        request: Request<PriceRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let ticker = request.into_inner().ticker.to_uppercase();
        
        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            return Err(Status::invalid_argument(format!("Invalid ticker: {}", ticker)));
        }

        let (_, price) = crate::utils::generate_random_ticker_and_price();
        let formatted_message = crate::utils::format_price(&ticker, price);

        let mut tracker = self.price_tracker.lock().await;
        tracker.add_price(&ticker, price);

        Ok(Response::new(PriceResponse {
            ticker,
            price,
            formatted_message,
        }))
    }

    async fn get_stats(
        &self,
        request: Request<StatsRequest>,
    ) -> Result<Response<StatsResponse>, Status> {
        let ticker = request.into_inner().ticker.to_uppercase();
        
        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            return Err(Status::invalid_argument(format!("Invalid ticker: {}", ticker)));
        }

        let tracker = self.price_tracker.lock().await;
        
        let prices = match tracker.get_prices(&ticker) {
            Some(p) => p.clone(),
            None => return Err(Status::not_found(format!("No data available for ticker: {}", ticker))),
        };

        let average = tracker.average(&ticker).unwrap_or(0.0);
        let std_deviation = tracker.std_deviation(&ticker).unwrap_or(0.0);
        
        let formatted_message = format!(
            "Stats for {}:\nPrices: {:?}\nAverage: {:.2}\nStd Dev: {:.2}\n",
            ticker, prices, average, std_deviation
        );

        Ok(Response::new(StatsResponse {
            ticker,
            prices,
            average,
            std_deviation,
            formatted_message,
        }))
    }

    type StreamPricesStream = Pin<Box<dyn Stream<Item = Result<PriceResponse, Status>> + Send + 'static>>;

    async fn stream_prices(
        &self,
        request: Request<PriceRequest>,
    ) -> Result<Response<Self::StreamPricesStream>, Status> {
        let ticker = request.into_inner().ticker.to_uppercase();
        
        if !crate::utils::TICKERS.contains(&ticker.as_str()) {
            return Err(Status::invalid_argument(format!("Invalid ticker: {}", ticker)));
        }

        let (tx, rx) = mpsc::channel(32);
        let price_tracker = self.price_tracker.clone();

        // Spawn a task that will generate prices and send them through the channel
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                let (_, price) = crate::utils::generate_random_ticker_and_price();
                let formatted_message = crate::utils::format_price(&ticker, price);

                {
                    let mut tracker = price_tracker.lock().await;
                    tracker.add_price(&ticker, price);
                }

                if tx.send(Ok(PriceResponse {
                    ticker: ticker.clone(),
                    price,
                    formatted_message,
                }))
                .await
                .is_err()
                {
                    break;
                }
            }
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::StreamPricesStream))
    }
}

pub async fn run_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port).parse()?;
    let service = StockServiceImpl::new();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(StockServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
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
