use crate::utils::PriceTracker;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct StockServiceImpl {
    pub(crate) price_tracker: Arc<Mutex<PriceTracker>>,
    pub(crate) active_clients: Arc<Mutex<HashMap<SocketAddr, SystemTime>>>,
}

impl StockServiceImpl {
    pub fn new() -> Self {
        StockServiceImpl {
            price_tracker: Arc::new(Mutex::new(PriceTracker::new())),
            active_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_client(&self, addr: SocketAddr) {
        let mut clients = self.active_clients.lock().await;
        if !clients.contains_key(&addr) {
            println!(
                "Client connected from {}. Active clients: {}",
                addr,
                clients.len() + 1
            );
            clients.insert(addr, SystemTime::now());
        }
    }

    pub async fn update_last_activity(&self, addr: Option<SocketAddr>) {
        if let Some(addr) = addr {
            let mut clients = self.active_clients.lock().await;
            clients.insert(addr, SystemTime::now());
        }
    }

    pub async fn get_active_client_count(&self) -> usize {
        let mut clients = self.active_clients.lock().await;
        // Remove clients that haven't been active for more than 30 seconds
        clients.retain(|_, last_active| {
            last_active
                .elapsed()
                .map(|elapsed| elapsed < Duration::from_secs(30))
                .unwrap_or(false)
        });
        clients.len()
    }
}
