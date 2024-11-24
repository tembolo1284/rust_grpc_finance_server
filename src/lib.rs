pub mod client;
pub mod config;
pub mod server;
pub mod utils;

// This will include the generated protobuf code
pub mod finance {
    include!(concat!(env!("OUT_DIR"), "/finance.rs"));
}
