use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;

pub struct Ibeacon {
    config: CollectorCfg,
}

pub struct IbeaconFactory {
    config: CollectorCfg,
}

impl IbeaconFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for IbeaconFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Ibeacon { config: self.config.clone() })
    }
}

impl Collector for Ibeacon {
    fn name(&self) -> &'static str {
        "ibeacon"
    }

    fn is_enable(&self) -> bool {
        self.config.mqtt.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let config = self.config.ibeacon.clone();
        Ok(())
    }
}