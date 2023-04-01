use rumqttd::{Broker, Config, Notification};
use std::thread;

pub struct Mqtt {
    pub config: CollectorConfig,
}

use crate::config::CollectorConfig;

use super::Collector;
use super::grpc;

impl Collector for Mqtt {
    fn name(&self) -> &str {
        "mqtt"
    }
    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let mut mqtt_config: Config = confy::load_path("config/mqttd.conf").unwrap();
        let mut server_settings = mqtt_config.v4.get("1").unwrap().clone();
        server_settings.listen = self.config.mqtt.host.parse().unwrap();
        mqtt_config.v4.insert("1".to_string(), server_settings.to_owned());
        let listening_host = &mqtt_config.v4.get("1").unwrap().listen.to_owned();
        let mut broker = Broker::new(mqtt_config);
        let (mut tx, mut rx) = broker.link("kraken").unwrap();
        thread::spawn(move || {
            match broker.start() {
                Ok(_) => debug!("MQTT Broker was started."),
                Err(e) => error!("Failed to start MQTT Broker: {}", e),
            }
        });
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
                    let sent = grpc::send(&message, &"mqtt").await;
                    match sent {
                        Ok(msg) => debug!("Sent message to grpc server: {:?}", msg),
                        Err(msg) => error!("Failed to send to grpc: {:?}", msg),
                    }
                }
                v => {
                    debug!("{v:?}");
                    continue;
                }
            };
        }
    }
}