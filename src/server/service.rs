use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use tokio::sync::watch;
use crate::utils::PriceTracker;

#[derive(Clone)]
pub struct StockServiceImpl {
    pub(crate) price_tracker: Arc<Mutex<PriceTracker>>,
    pub(crate) active_clients: Arc<AtomicUsize>,
    pub(crate) total_connections: Arc<AtomicUsize>,
    pub(crate) shutdown_sender: watch::Sender<bool>,
}

impl StockServiceImpl {
    pub fn new(shutdown_sender: watch::Sender<bool>) -> Self {
        StockServiceImpl {
            price_tracker: Arc::new(Mutex::new(PriceTracker::new())),
            active_clients: Arc::new(AtomicUsize::new(0)),
            total_connections: Arc::new(AtomicUsize::new(0)),
            shutdown_sender,
        }
    }

    pub fn increment_clients(&self) {
        self.active_clients.fetch_add(1, Ordering::SeqCst);
        let total = self.total_connections.fetch_add(1, Ordering::SeqCst);
        println!(
            "Client connected. Active clients: {}, Total connections: {}",
            self.active_clients.load(Ordering::SeqCst),
            total + 1
        );
    }

    pub fn decrement_clients(&self) {
        let active = self.active_clients.fetch_sub(1, Ordering::SeqCst);
        let total = self.total_connections.load(Ordering::SeqCst);
        println!(
            "Client disconnected. Active clients: {}, Total connections: {}",
            active - 1,
            total
        );

        if active == 1 && total >= 2 {
            println!("All clients disconnected and minimum connection threshold met. Initiating shutdown...");
            let _ = self.shutdown_sender.send(true);
        }
    }
}
