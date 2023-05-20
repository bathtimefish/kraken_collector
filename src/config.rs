use std::env;

#[derive (Clone, Debug)]
pub struct WebhookCfg {
    pub enable: bool,
    pub path: String,
    pub port: u16,
}

#[derive (Clone, Debug)]
pub struct MqttCfg {
    pub enable: bool,
    pub host: String,
    pub topic: String,
    pub config_path: String,
}

#[derive (Clone, Debug)]
pub struct WebsocketCfg {
    pub enable: bool,
    pub host: String,
    pub sub_protocol: String,
}

#[derive (Clone, Debug)]
pub struct GrpcCfg {
    pub host: String,
}


#[derive (Clone, Debug)]
pub struct CollectorCfg {
    pub webhook: WebhookCfg,
    pub mqtt: MqttCfg,
    pub websocket: WebsocketCfg,
    pub grpc: GrpcCfg,
}

impl Default for CollectorCfg {
    fn default() -> Self {
        let mut webhook_enable = false;
        let mut mqtt_enable = false;
        let mut websocket_enable = false;
        if env::var("KRKNC_WEBHOOK_PATH").is_ok() {
            webhook_enable = true;
        }
        if env::var("KRKNC_MQTT_HOST").is_ok() {
            mqtt_enable = true;
        }
        if env::var("KRKNC_WEBSOCKET_HOST").is_ok() {
            websocket_enable = true;
        }
        CollectorCfg {
            grpc: GrpcCfg {
                host: env::var("KRKNC_BROKER_HOST").unwrap_or("http://[::1]:50051".to_string()),
            },
            webhook: WebhookCfg {
                enable: webhook_enable,
                path: env::var("KRKNC_WEBHOOK_PATH").unwrap_or("/webhook".to_string()),
                port: env::var("KRKNC_WEBHOOK_PORT").unwrap_or("2792".to_string()).parse::<u16>().unwrap(),
            },
            mqtt: MqttCfg {
                enable: mqtt_enable,
                host: env::var("KRKNC_MQTT_HOST").unwrap_or("127.0.0.1:1883".to_string()),
                topic: env::var("KRKNC_MQTT_TOPIC").unwrap_or("kraken".to_string()),
                config_path: env::var("KRKNC_MQTT_CONFIG_PATH").unwrap_or("config/mqttd.conf".to_string()),
            },
            websocket: WebsocketCfg {
                enable: websocket_enable,
                host: env::var("KRKNC_WEBSOCKET_HOST").unwrap_or("127.0.0.1:2794".to_string()),
                sub_protocol: env::var("KRKNC_WEBSOCKET_SUB_PROTOCOL").unwrap_or("kraken-ws".to_string()),
            },
        }
    }
}