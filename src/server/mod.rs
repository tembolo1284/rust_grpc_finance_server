use std::pin::Pin;
use futures::Stream;
use tonic::{Response, Status, transport::Server, Request, service::Interceptor};
use tokio::sync::watch;

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
        if let Some(remote_addr) = request.remote_addr() {
            let service = self.service.clone();
            
            // Register the client
            tokio::spawn(async move {
                if !service.is_client_registered(remote_addr).await {
                    service.register_client(remote_addr).await;
                    
                    // Set up connection monitoring
                    let service_clone = service.clone();
                    tokio::spawn(async move {
                        // Wait a short time for any potential immediate disconnection
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        
                        // Monitor for client disconnection
                        loop {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            
                            // Try to connect to client address to check if it's still alive
                            if tokio::net::TcpStream::connect(remote_addr).await.is_err() {
                                println!("Detected client disconnection: {}", remote_addr);
                                service_clone.unregister_client(remote_addr).await;
                                break;
                            }
                        }
                    });
                }
            });
        }
        Ok(request)
    }
}

pub async fn run_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port).parse()?;
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
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
        if let Some(remote_addr) = request.remote_addr() {
            let service = self.clone();
            tokio::spawn(async move {
                service.unregister_client(remote_addr).await;
            });
        }
        self.handle_stream_prices(request).await
    }
}
