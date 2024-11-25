pub mod client;
pub mod config;
pub mod server;
pub mod utils;

// Include the generated protobuf code
pub mod finance {
    tonic::include_proto!("finance");
}
