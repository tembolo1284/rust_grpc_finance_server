use futures::Stream;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use crate::finance::PriceResponse;
use super::service::StockServiceImpl;

impl StockServiceImpl {
    pub(crate) async fn handle_stream_prices(
        &self,
        request: Request<crate::finance::PriceRequest>,
    ) -> Result<Response<Pin<Box<dyn Stream<Item = Result<PriceResponse, Status>> + Send + 'static>>>, Status> {
        let remote_addr = request
            .remote_addr()
            .unwrap_or_else(|| "unknown".parse().unwrap());
            
        let ticker = request.into_inner().ticker.to_uppercase();
        println!(
            "Received streaming request for ticker: {} from {}",
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
        let service_clone = self.clone();

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
                    if let Ok(addr) = remote_addr.to_string().parse() {
                        service_clone.unregister_client(addr).await;
                    }
                    break;
                }
            }
        });

        println!("Established price stream for ticker: {}", stream_ticker);
        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Pin<Box<dyn Stream<Item = Result<PriceResponse, Status>> + Send + 'static>>
        ))
    }
}
