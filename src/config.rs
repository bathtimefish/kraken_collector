use std::env;

#[derive (Clone, Debug)]
pub struct WebhookCfg {
    pub enable: bool,
    #[allow(dead_code)]
    pub path: String,
    pub port: u16,
}

#[derive (Clone, Debug)]
pub struct MqttCfg {
    pub enable: bool,
    //pub host: String,
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
pub struct IbeaconCfg {
    pub enable: bool,
    pub filter_duration: u64,
    pub allowed_uuid_filter_path: String,
}

#[derive (Clone, Debug)]
pub struct SerialCfg {
    pub enable: bool,
    pub device_name: String,
    pub port: String,
    pub baudrate: u32,
    pub timeout: u64,
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
    pub ibeacon: IbeaconCfg,
    pub serial: SerialCfg,
}

impl Default for CollectorCfg {
    fn default() -> Self {
        let mut webhook_enable = false;
        let mut mqtt_enable = false;
        let mut websocket_enable = false;
        let mut ibeacon_enable = false;
        let mut serial_enable = false;
        if env::var("KRKNC_WEBHOOK_PATH").is_ok() {
            webhook_enable = true;
        }
        if env::var("KRKNC_MQTT_CONFIG_PATH").is_ok() {
            mqtt_enable = true;
        }
        if env::var("KRKNC_WEBSOCKET_HOST").is_ok() {
            websocket_enable = true;
        }
        if env::var("KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH").is_ok() {
            ibeacon_enable = true;
        }
        if env::var("KRKNC_SERIAL_DEVICE_NAME").is_ok() {
            serial_enable = true;
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
                //host: env::var("KRKNC_MQTT_HOST").unwrap_or("127.0.0.1:1883".to_string()),
                topic: env::var("KRKNC_MQTT_TOPIC").unwrap_or("kraken".to_string()),
                config_path: env::var("KRKNC_MQTT_CONFIG_PATH").unwrap_or("config/mqttd.conf".to_string()),
            },
            websocket: WebsocketCfg {
                enable: websocket_enable,
                host: env::var("KRKNC_WEBSOCKET_HOST").unwrap_or("127.0.0.1:2794".to_string()),
                sub_protocol: env::var("KRKNC_WEBSOCKET_SUB_PROTOCOL").unwrap_or("kraken-ws".to_string()),
            },
            ibeacon: IbeaconCfg {
                enable: ibeacon_enable,
                filter_duration: env::var("KRKNC_IBEACON_FILTER_DURATION").unwrap_or("1".to_string()).parse::<u64>().unwrap(),
                allowed_uuid_filter_path: env::var("KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH").unwrap_or("config/allowed_uuids.yml".to_string()),
            },
            serial: SerialCfg {
                enable: serial_enable,
                device_name: env::var("KRKNC_SERIAL_DEVICE_NAME").unwrap_or("unknown".to_string()),
                port: env::var("KRKNC_SERIAL_PORT").unwrap_or("/dev/ttyACM0".to_string()),
                baudrate: env::var("KRKNC_SERIAL_BAUDRATE").unwrap_or("9600".to_string()).parse::<u32>().unwrap(),
                timeout: env::var("KRKNC_SERIAL_TIMEOUT_SEC").unwrap_or("10".to_string()).parse::<u64>().unwrap(),
            },
        }
    }
}
