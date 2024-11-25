use futures::Stream;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;
use tonic::{service::Interceptor, transport::Server, Request, Response, Status};

mod handlers;
mod service;
mod stream;

pub use service::StockServiceImpl;

#[derive(Clone)]
struct ConnectionInterceptor {
    service: StockServiceImpl,
}

impl Interceptor for ConnectionInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(remote_addr) = request.remote_addr() {
            let service = self.service.clone();
            // Register the client
            tokio::spawn(async move {
                service.register_client(remote_addr).await;
            });
        }
        Ok(request)
    }
}

pub async fn run_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port).parse()?;
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    let service = StockServiceImpl::new();
    println!("Server starting up...");
    println!("Server listening on {}", addr);

    let service_for_monitoring = service.clone();
    let service_for_interceptor = service.clone();
    // Spawn a task to monitor client activity
    tokio::spawn(async move {
        let mut inactivity_count = 0u32;
        loop {
            sleep(Duration::from_secs(5)).await;
            let active_count = service_for_monitoring.get_active_client_count().await;

            if active_count == 0 {
                inactivity_count += 1;
                if inactivity_count >= 2 {
                    // 10 seconds of no activity
                    println!("No active clients for 10 seconds. Initiating shutdown...");
                    let _ = shutdown_tx.send(true);
                    break;
                }
            } else {
                inactivity_count = 0;
            }
        }
    });

    let intercepted_service =
        crate::finance::stock_service_server::StockServiceServer::with_interceptor(
            service,
            ConnectionInterceptor {
                service: service_for_interceptor,
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
        self.update_last_activity(request.remote_addr()).await;
        self.handle_get_ticker_list(request).await
    }

    async fn get_price(
        &self,
        request: Request<crate::finance::PriceRequest>,
    ) -> Result<Response<crate::finance::PriceResponse>, Status> {
        self.update_last_activity(request.remote_addr()).await;
        self.handle_get_price(request).await
    }

    async fn get_multiple_prices(
        &self,
        request: Request<crate::finance::MultiplePricesRequest>,
    ) -> Result<Response<crate::finance::MultiplePricesResponse>, Status> {
        self.update_last_activity(request.remote_addr()).await;
        self.handle_get_multiple_prices(request).await
    }

    async fn get_stats(
        &self,
        request: Request<crate::finance::StatsRequest>,
    ) -> Result<Response<crate::finance::StatsResponse>, Status> {
        self.update_last_activity(request.remote_addr()).await;
        self.handle_get_stats(request).await
    }

    type StreamPricesStream =
        Pin<Box<dyn Stream<Item = Result<crate::finance::PriceResponse, Status>> + Send + 'static>>;

    async fn stream_prices(
        &self,
        request: Request<crate::finance::PriceRequest>,
    ) -> Result<Response<Self::StreamPricesStream>, Status> {
        self.update_last_activity(request.remote_addr()).await;
        self.handle_stream_prices(request).await
    }
}
