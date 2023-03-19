use std::thread;
use crate::collectors::{
    Collector,
    webhook::Webhook,
    mqtt::Mqtt,
    websocket::Websocket,
};

pub fn start() {
    thread::spawn(move || {
        let webhook: Webhook = Collector::new();
        let started = webhook.start();
        match started {
            Ok(_) => debug!("Webhook collector started."),
            Err(e) => error!("Failed to start webhook collector: {}", e),
        }
    });
    thread::spawn(move || {
        let mqtt: Mqtt = Collector::new();
        let started = mqtt.start();
        match started {
            Ok(_) => debug!("MQTT collector started."),
            Err(e) => error!("Failed to start MQTT collector: {}", e),
        }
    });
    thread::spawn(move || {
        let websocket: Websocket = Collector::new();
        let started = websocket.start();
        match started {
            Ok(_) => debug!("Websocket collector started."),
            Err(e) => error!("Failed to start websocket collector: {}", e),
        }
    });

    debug!("collector service started.");
    loop {}
}