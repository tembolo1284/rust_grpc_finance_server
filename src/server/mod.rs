use std::pin::Pin;
use futures::Stream;
use tonic::{Response, Status, transport::Server, Request, service::Interceptor};

mod service;
mod handlers;
mod stream;

pub use service::StockServiceImpl;

#[derive(Clone)]
struct ConnectionInterceptor {
    service: StockServiceImpl,
}

impl Interceptor for ConnectionInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        self.service.increment_clients();
        Ok(request)
    }
}

impl Drop for ConnectionInterceptor {
    fn drop(&mut self) {
        self.service.decrement_clients();
    }
}

pub async fn run_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port).parse()?;
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    let service = StockServiceImpl::new(shutdown_tx);
    
    println!("Server starting up...");
    println!("Server listening on {}", addr);

    let intercepted_service = crate::finance::stock_service_server::StockServiceServer::with_interceptor(
        service.clone(),
        ConnectionInterceptor {
            service: service.clone(),
        },
    );

    let server = Server::builder()
        .add_service(intercepted_service)
        .serve_with_shutdown(addr, async move {
            shutdown_rx.changed().await.ok();
            println!("Initiating graceful shutdown...");
        });

    println!("Server is ready to accept connections");
    server.await?;
    println!("Server has shut down gracefully");

    Ok(())
}

#[tonic::async_trait]
impl crate::finance::stock_service_server::StockService for StockServiceImpl {
    async fn get_ticker_list(
        &self,
        request: Request<crate::finance::TickerListRequest>,
    ) -> Result<Response<crate::finance::TickerListResponse>, Status> {
        self.handle_get_ticker_list(request).await
    }

    async fn get_price(
        &self,
        request: Request<crate::finance::PriceRequest>,
    ) -> Result<Response<crate::finance::PriceResponse>, Status> {
        self.handle_get_price(request).await
    }

    async fn get_multiple_prices(
        &self,
        request: Request<crate::finance::MultiplePricesRequest>,
    ) -> Result<Response<crate::finance::MultiplePricesResponse>, Status> {
        self.handle_get_multiple_prices(request).await
    }

    async fn get_stats(
        &self,
        request: Request<crate::finance::StatsRequest>,
    ) -> Result<Response<crate::finance::StatsResponse>, Status> {
        self.handle_get_stats(request).await
    }

    type StreamPricesStream = Pin<Box<dyn Stream<Item = Result<crate::finance::PriceResponse, Status>> + Send + 'static>>;

    async fn stream_prices(
        &self,
        request: Request<crate::finance::PriceRequest>,
    ) -> Result<Response<Self::StreamPricesStream>, Status> {
        self.handle_stream_prices(request).await
    }
}
