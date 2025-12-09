use rumqttd::{Broker, Config, Notification};
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;

pub struct Mqtt {
    config: CollectorCfg,
}

pub struct MqttFactory {
    config: CollectorCfg,
}

impl MqttFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for MqttFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Mqtt { config: self.config.clone() })
    }
}

impl Collector for Mqtt {
    fn name(&self) -> &'static str {
        "mqtt"
    }

    fn is_enable(&self) -> bool {
        self.config.mqtt.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(&self.config.mqtt.config_path.to_owned()))
            .build()
            .unwrap();
        let config: Config = config.try_deserialize().unwrap();
        let config_for_info = config.clone();
        let mut broker = Broker::new(config);

        let (mut tx, mut rx) = broker.link("kraken").unwrap();
        
        std::thread::spawn(move || {
            if let Err(e) = broker.start() {
                error!("Failed to start MQTT Broker: {}", e);
            } else {
                debug!("MQTT Broker was started.");
            }
        });
        
        tx.subscribe(&self.config.mqtt.topic).unwrap();

        // Log TCP MQTT v4 endpoint
        if let Some(server) = config_for_info.v4.as_ref().and_then(|v4| v4.get("1")) {
            debug!("MQTT Broker was started that is listening on {} (TCP v4)", server.listen.to_string());
        }

        // Log TCP MQTT v5 endpoint
        if let Some(server) = config_for_info.v5.as_ref().and_then(|v5| v5.get("1")) {
            debug!("MQTT Broker was started that is listening on {} (TCP v5)", server.listen.to_string());
        }

        // Log WebSocket endpoint
        if let Some(ws_server) = config_for_info.ws.as_ref().and_then(|ws| ws.get("1")) {
            debug!("MQTT Broker was started that is listening on {} (WebSocket)", ws_server.listen.to_string());
        }

        loop {
            if let Some(notification) = rx.recv().unwrap() {
                match notification {
                    Notification::Forward(forward) => {
                        debug!("Forward: {:?}", forward);
                        let message = String::from_utf8_lossy(&forward.publish.payload);
                        let sent = grpc::send(
                            &self.config.grpc,
                            "mqtt",
                            "application/json",
                            "{}",
                            message.as_bytes(),
                        ).await;
                        if let Err(e) = sent {
                            error!("Failed to send to grpc: {:?}", e);
                        } else {
                            debug!("Sent message to grpc server: {:?}", sent);
                        }
                    }
                    v => {
                        trace!("{:?}", v);
                    }
                }
            } else {
                error!("MQTT Broker disconnected");
            }
        }
    }
}

