use std::env;

#[derive (Clone, Debug)]
pub struct WebhookCfg {
    pub path: String,
    pub port: u16,
}

#[derive (Clone, Debug)]
pub struct MqttCfg {
    pub host: String,
    pub topic: String,
}

#[derive (Clone, Debug)]
pub struct WebsocketCfg {
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
        CollectorCfg {
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
        }
    }
}