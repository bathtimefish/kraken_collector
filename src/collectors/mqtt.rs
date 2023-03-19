use rumqttd::{Broker, Config, Notification};
use std::thread;

pub(crate) struct Mqtt;

use super::Collector;
use super::grpc;

impl Collector for Mqtt {
    fn new() -> Self {
        Mqtt {}
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let config: Config = confy::load_path("config/mqttd.conf").unwrap();
        let listening_host = &config.v4.get("1").unwrap().listen.to_owned();
        let mut broker = Broker::new(config);
        let (mut tx, mut rx) = broker.link("kraken").unwrap();
        thread::spawn(move || {
            match broker.start() {
                Ok(_) => debug!("MQTT Broker was started."),
                Err(e) => error!("Failed to start MQTT Broker: {}", e),
            }
        });
        tx.subscribe("kraken").unwrap();
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