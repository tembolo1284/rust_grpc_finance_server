pub mod utils;
pub mod client;
pub mod server;
pub mod config;

// This will include the generated protobuf code
pub mod finance {
    include!(concat!(env!("OUT_DIR"), "/finance.rs"));
}
