
pub trait Collector {
    fn name(&self) -> &str;
    fn start(&self) -> Result<(), anyhow::Error>;
}

pub mod grpc;
pub mod webhook;
pub mod mqtt;
pub mod websocket;
