use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashSet;
use tokio::sync::Mutex;
use tokio::sync::watch;
use std::net::SocketAddr;
use crate::utils::PriceTracker;

#[derive(Clone)]
pub struct StockServiceImpl {
    pub(crate) price_tracker: Arc<Mutex<PriceTracker>>,
    pub(crate) active_clients: Arc<Mutex<HashSet<SocketAddr>>>,
    pub(crate) total_connections: Arc<AtomicUsize>,
    pub(crate) shutdown_sender: watch::Sender<bool>,
}

impl StockServiceImpl {
    pub fn new(shutdown_sender: watch::Sender<bool>) -> Self {
        StockServiceImpl {
            price_tracker: Arc::new(Mutex::new(PriceTracker::new())),
            active_clients: Arc::new(Mutex::new(HashSet::new())),
            total_connections: Arc::new(AtomicUsize::new(0)),
            shutdown_sender,
        }
    }

    pub async fn register_client(&self, addr: SocketAddr) {
        let mut clients = self.active_clients.lock().await;
        if clients.insert(addr) {
            let total = self.total_connections.fetch_add(1, Ordering::SeqCst);
            println!(
                "Client connected from {}. Active clients: {}, Total connections: {}",
                addr,
                clients.len(),
                total + 1
            );
        }
    }

    pub async fn unregister_client(&self, addr: SocketAddr) {
        let mut clients = self.active_clients.lock().await;
        if clients.remove(&addr) {
            let total = self.total_connections.load(Ordering::SeqCst);
            println!(
                "Client disconnected from {}. Active clients: {}, Total connections: {}",
                addr,
                clients.len(),
                total
            );

            if clients.is_empty() && total >= 2 {
                println!("All clients disconnected and minimum connection threshold met. Initiating shutdown...");
                let _ = self.shutdown_sender.send(true);
            }
        }
    }
    
    pub async fn is_client_registered(&self, addr: SocketAddr) -> bool {
        let clients = self.active_clients.lock().await;
        clients.contains(&addr)
    }
}
