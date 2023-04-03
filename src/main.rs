#[macro_use]
extern crate log;
use std::env;
use anyhow::Result;

mod service;
pub mod collectors;
pub mod config;
use crate::config::{
    CollectorCfg,
    WebhookCfg,
    MqttCfg,
    WebsocketCfg,
    GrpcCfg,
};

fn main() -> Result<(), anyhow::Error>{
    /* get env values */
    let config = CollectorCfg {
        grpc: GrpcCfg {
            host: env::var("KRKNC_BROKER_HOST").unwrap_or("http://[::1]:50051".to_string()),
        },
        webhook: WebhookCfg {
            path: env::var("KRKNC_WEBHOOK_PATH").unwrap_or("/webhook".to_string()),
            port: env::var("KRKNC_WEBHOOK_PORT").unwrap_or("2792".to_string()).parse::<u16>().unwrap(),
        },
        mqtt: MqttCfg {
            host: env::var("KRKNC_MQTT_HOST").unwrap_or("127.0.0.1:1883".to_string()),
            topic: env::var("KRKNC_MQTT_TOPIC").unwrap_or("kraken".to_string()),
        },
        websocket: WebsocketCfg {
            host: env::var("KRKNC_WEBSOCKET_HOST").unwrap_or("127.0.0.1:2794".to_string()),
            sub_protocol: env::var("KRKNC_WEBSOCKET_SUB_PROTOCOL").unwrap_or("kraken-ws".to_string()),
        },
    };
    let log_level = env::var("LOG_LEVEL").unwrap_or("info".to_string());
    env_logger::init();
    info!("RUST_ROG: {}", log_level);
    info!("KRAKEN Collector -- The Highlevel Data Collector -- boot squence start.");
    service::start(config).unwrap();
    Ok(())
}
