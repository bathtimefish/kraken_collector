use crate::{
    collectors::{
        Collector,
        webhook::Webhook,
        mqtt::Mqtt,
        websocket::Websocket,
    },
    config::CollectorConfig
};

// #[tokio::main(flavor = "multi_thread", worker_threads = 3)]
#[tokio::main(flavor = "current_thread")]
pub async fn start(config: CollectorConfig) {
    let webhook = Box::new(Webhook{ config: config.clone() });
    let mqtt = Box::new(Mqtt{ config: config.clone() });
    let websocket = Box::new(Websocket{ config: config.clone() });

    std::thread::spawn(move || {
        let started = webhook.start();
        match started {
            Ok(_) => debug!("Webhook collector started."),
            Err(e) => error!("Failed to start webhook collector: {}", e),
        }
    });
    std::thread::spawn(move || {
        let started = mqtt.start();
        match started {
            Ok(_) => debug!("MQTT collector started."),
            Err(e) => error!("Failed to start MQTT collector: {}", e),
        }
    });
    std::thread::spawn(move || {
        let started = websocket.start();
        match started {
            Ok(_) => debug!("Websocket collector started."),
            Err(e) => error!("Failed to start websocket collector: {}", e),
        }
    });

    debug!("collector service started.");
    loop {}
}