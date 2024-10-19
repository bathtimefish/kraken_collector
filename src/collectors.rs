pub trait Collector: Send {
    fn name(&self) -> &'static str;
    fn is_enable(&self) -> bool;
    fn start(&self) -> Result<(), anyhow::Error>;
}

pub trait CollectorFactory: Send {
    fn create(&self) -> Box<dyn Collector>;
}


pub mod grpc;
pub mod webhook;
pub mod mqtt;
pub mod websocket;
pub mod ibeacon;
