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
pub struct CollectorConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub webhook: WebhookCfg,
    pub mqtt: MqttCfg,
    pub websocket: WebsocketCfg,
}