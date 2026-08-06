pub trait Collector: Send {
    fn name(&self) -> &'static str;
    fn is_enable(&self) -> bool;
    fn start(&self) -> Result<(), anyhow::Error>;
}

pub trait CollectorFactory: Send {
    fn create(&self) -> Box<dyn Collector>;
}

#[cfg(feature = "bjig")]
pub mod bjig;
pub mod camera;
#[cfg(feature = "direct4b")]
pub mod direct4b;
pub mod email;
pub mod grpc;
pub mod ibeacon;
pub mod mqtt;
pub mod serial;
pub mod tcp;
pub mod textfile;
pub mod webhook;
pub mod websocket;
