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