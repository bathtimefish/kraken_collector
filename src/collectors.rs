pub trait Collector {
    fn new() -> Self;
    fn start(&self) -> Result<(), anyhow::Error>;
}

pub mod grpc;
pub mod webhook;
pub mod mqtt;
pub mod websocket;
