pub trait Collector: Send {
    fn name(&self) -> &'static str;
    fn start(&self) -> Result<(), anyhow::Error>;
}

pub trait CollectorFactory: Send {
    fn create(&self) -> Box<dyn Collector>;
}

pub mod grpc;
pub mod webhook;
pub mod mqtt;
pub mod websocket;
