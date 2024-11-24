use rand::Rng;
use std::collections::HashMap;

pub static TICKERS: &[&str] = &[
    "AAPL",
    "MSFT",
    "GOOG",
    "AMZN",
    "META",
    "NFLX",
    "TSLA",
    "NVDA",
    "AMD",
    "INTC",
];

pub fn generate_random_ticker_and_price() -> (String, f64) {
    let mut rng = rand::thread_rng();
    let ticker = TICKERS[rng.gen_range(0..TICKERS.len())];
    let price = rng.gen_range(10.0..1000.0);
    (ticker.to_string(), price)
}

pub fn format_price(ticker: &str, price: f64) -> String {
    format!("Current price for {}: ${:.2}\n", ticker, price)
}

#[derive(Default)]
pub struct PriceTracker {
    prices: HashMap<String, Vec<f64>>,
}

impl PriceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_price(&mut self, ticker: &str, price: f64) {
        self.prices
            .entry(ticker.to_string())
            .or_default()
            .push(price);
    }

    pub fn get_prices(&self, ticker: &str) -> Option<&Vec<f64>> {
        self.prices.get(ticker)
    }

    pub fn average(&self, ticker: &str) -> Option<f64> {
        self.get_prices(ticker).map(|prices| {
            if prices.is_empty() {
                0.0
            } else {
                prices.iter().sum::<f64>() / prices.len() as f64
            }
        })
    }

    pub fn std_deviation(&self, ticker: &str) -> Option<f64> {
        self.get_prices(ticker).map(|prices| {
            if prices.is_empty() {
                0.0
            } else {
                let avg = prices.iter().sum::<f64>() / prices.len() as f64;
                let variance = prices
                    .iter()
                    .map(|value| {
                        let diff = value - avg;
                        diff * diff
                    })
                    .sum::<f64>()
                    / prices.len() as f64;
                variance.sqrt()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_tracker() {
        let mut tracker = PriceTracker::new();
        let ticker = "AAPL";

        // Test adding prices
        tracker.add_price(ticker, 150.0);
        tracker.add_price(ticker, 160.0);
        tracker.add_price(ticker, 170.0);

        // Test getting prices
        assert_eq!(tracker.get_prices(ticker), Some(&vec![150.0, 160.0, 170.0]));

        // Test average
        assert_eq!(tracker.average(ticker), Some(160.0));

        // Test standard deviation
        let std_dev = tracker.std_deviation(ticker).unwrap();
        assert!((std_dev - 8.16496580927726).abs() < 0.000001);
    }

    #[test]
    fn test_format_price() {
        assert_eq!(
            format_price("AAPL", 150.50),
            "Current price for AAPL: $150.50\n"
        );
    }

    #[test]
    fn test_random_ticker_and_price() {
        let (ticker, price) = generate_random_ticker_and_price();
        assert!(TICKERS.contains(&ticker.as_str()));
        assert!(price >= 10.0 && price < 1000.0);
    }
}
