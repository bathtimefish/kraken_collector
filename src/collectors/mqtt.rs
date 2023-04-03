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
        Box::new(Mqtt{ config: self.config.clone() })
    }
}

impl Collector for Mqtt {
    fn name(&self) -> &'static str {
        "mqtt"
    }
    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let mut mqtt_config: Config = confy::load_path("config/mqttd.conf").unwrap();
        let mut server_settings = mqtt_config.v4.get("1").unwrap().clone();
        let config = self.config.mqtt.clone();
        server_settings.listen = config.host.parse().unwrap();
        mqtt_config.v4.insert("1".to_string(), server_settings.to_owned());
        let listening_host = &mqtt_config.v4.get("1").unwrap().listen.to_owned();
        let mut broker = Broker::new(mqtt_config);
        let (mut tx, mut rx) = broker.link("kraken").unwrap();
        std::thread::spawn(
            move || {
                match broker.start() {
                    Ok(_) => debug!("MQTT Broker was started."),
                    Err(e) => error!("Failed to start MQTT Broker: {}", e),
                }
            }
        );
        tx.subscribe(&self.config.mqtt.topic).unwrap();
        debug!("MQTT Broker was started that is listening on {}", listening_host.to_string());
        loop {
            let notification = match rx.recv().unwrap() {
                Some(notification) => notification,
                None => {
                    error!("MQTT Broker disconnected");
                    continue;
                }
            };
            match notification {
                Notification::Forward(forward) => {
                    debug!("Forward: {:?}", forward);
                    let message = String::from_utf8_lossy(&forward.publish.payload);
                    let sent = grpc::send(&self.config.grpc, &message, &"mqtt").await;
                    match sent {
                        Ok(msg) => debug!("Sent message to grpc server: {:?}", msg),
                        Err(msg) => error!("Failed to send to grpc: {:?}", msg),
                    }
                }
                v => {
                    trace!("{v:?}");
                    continue;
                }
            };
        }
    }
}