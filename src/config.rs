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
pub struct TextFileCfg {
    pub enable: bool,
    pub target_file_path: String,
    pub monitor_dir_path: String,
    pub interval_sec: u64,
    pub monitoring_mode: String,
    pub allow_create: bool,
    pub allow_modify: bool,
    pub remove_created: bool,
    pub remove_except_modified: bool,
    pub remove_all_files: bool,
    pub remove_all_folder: bool,
}

#[derive (Clone, Debug)]
pub struct CameraCfg {
    pub enable: bool,
    pub capture_interval_sec: u64,
}

#[derive (Clone, Debug)]
pub struct GrpcCfg {
    pub host: String,
}

#[derive (Clone, Debug)]
pub struct EmailCfg {
    pub enable: bool,
    pub host_addr: String,
    pub smtp_port: u16,
    pub max_message_size: usize,
    pub max_attachment_size: usize,
    #[allow(dead_code)]
    pub domain: String,
    #[allow(dead_code)]
    pub auth_required: bool,
    pub allowed_senders: Vec<String>,
    // TLS settings (reserved for future implementation)
    #[allow(dead_code)]
    pub tls_enabled: bool,
    #[allow(dead_code)]
    pub tls_cert_path: Option<String>,
    #[allow(dead_code)]
    pub tls_key_path: Option<String>,
    #[allow(dead_code)]
    pub tls_require: bool,
}

#[derive (Clone, Debug)]
pub struct BjigCfg {
    pub enable: bool,
    pub device_path: String,
    pub cli_bin_path: String,
    pub data_timeout_sec: u64,
    pub action_cooldown_sec: u64,
}

impl Default for EmailCfg {
    fn default() -> Self {
        let mut email_enable = false;
        // Enable if any email-specific env var is set
        if env::var("KRKNC_EMAIL_HOST_ADDR").is_ok()
            || env::var("KRKNC_EMAIL_SMTP_PORT").is_ok() {
            email_enable = true;
        }

        let allowed_senders = env::var("KRKNC_EMAIL_ALLOWED_SENDERS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let tls_cert_path = env::var("KRKNC_EMAIL_TLS_CERT_PATH")
            .ok()
            .filter(|s| !s.is_empty());

        let tls_key_path = env::var("KRKNC_EMAIL_TLS_KEY_PATH")
            .ok()
            .filter(|s| !s.is_empty());

        EmailCfg {
            enable: email_enable,
            host_addr: env::var("KRKNC_EMAIL_HOST_ADDR")
                .unwrap_or("0.0.0.0".to_string()),
            smtp_port: env::var("KRKNC_EMAIL_SMTP_PORT")
                .unwrap_or("587".to_string())
                .parse::<u16>()
                .unwrap_or(587),
            max_message_size: env::var("KRKNC_EMAIL_MAX_MESSAGE_SIZE")
                .unwrap_or("10485760".to_string())
                .parse::<usize>()
                .unwrap_or(10485760), // 10MB
            max_attachment_size: env::var("KRKNC_EMAIL_MAX_ATTACHMENT_SIZE")
                .unwrap_or("5242880".to_string())
                .parse::<usize>()
                .unwrap_or(5242880), // 5MB
            domain: env::var("KRKNC_EMAIL_DOMAIN")
                .unwrap_or("localhost".to_string()),
            auth_required: env::var("KRKNC_EMAIL_AUTH_REQUIRED")
                .unwrap_or("false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            allowed_senders,
            tls_enabled: env::var("KRKNC_EMAIL_TLS_ENABLED")
                .unwrap_or("false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            tls_cert_path,
            tls_key_path,
            tls_require: env::var("KRKNC_EMAIL_TLS_REQUIRE")
                .unwrap_or("false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
        }
    }
}


#[derive (Clone, Debug)]
pub struct CollectorCfg {
    pub webhook: WebhookCfg,
    pub mqtt: MqttCfg,
    pub websocket: WebsocketCfg,
    pub grpc: GrpcCfg,
    pub ibeacon: IbeaconCfg,
    pub serial: SerialCfg,
    pub text_file: TextFileCfg,
    pub camera: CameraCfg,
    pub email: EmailCfg,
    pub bjig: BjigCfg,
}

impl Default for CollectorCfg {
    fn default() -> Self {
        let mut webhook_enable = false;
        let mut mqtt_enable = false;
        let mut websocket_enable = false;
        let mut ibeacon_enable = false;
        let mut serial_enable = false;
        let mut textfile_enable = false;
        let mut camera_enable = false;
        let mut bjig_enable = false;
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
        if env::var("KRKNC_TEXTFILE_MONITOR_DIR_PATH").is_ok() {
            textfile_enable = true;
        }
        if env::var("KRKNC_CAMERA_CAPTURE_INTERVAL_SEC").is_ok() {
            camera_enable = true;
        }
        if env::var("KRKNC_BJIG_DEVICE_PATH").is_ok() {
            bjig_enable = true;
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
            text_file: TextFileCfg {
                enable: textfile_enable,
                target_file_path: env::var("KRKNC_TEXTFILE_TARGET_FILE_PATH").unwrap_or("data/data.txt".to_string()),
                monitor_dir_path: env::var("KRKNC_TEXTFILE_MONITOR_DIR_PATH").unwrap_or("data/".to_string()),
                interval_sec: env::var("KRKNC_TEXTFILE_GET_INTERVAL_SEC").unwrap_or("10".to_string()).parse::<u64>().unwrap(),
                monitoring_mode: env::var("KRKNC_TEXTFILE_MONITORING_MODE").unwrap_or("time_interval".to_string()),
                allow_create: env::var("KRKNC_TEXTFILE_ALLOW_CREATE").unwrap_or("true".to_string()).parse::<bool>().unwrap(),
                allow_modify: env::var("KRKNC_TEXTFILE_ALLOW_MODIFY").unwrap_or("true".to_string()).parse::<bool>().unwrap(),
                remove_created: env::var("KRKNC_TEXTFILE_REMOVE_CREATED_FILE_AFTER_READ").unwrap_or("false".to_string()).parse::<bool>().unwrap(),
                remove_except_modified: env::var("KRKNC_TEXTFILE_REMOVE_FILES_EXCEPT_MODIFIED_AFTER_READ").unwrap_or("false".to_string()).parse::<bool>().unwrap(),
                remove_all_files: env::var("KRKNC_TEXTFILE_REMOVE_ALL_FILES_AFTER_READ").unwrap_or("false".to_string()).parse::<bool>().unwrap(),
                remove_all_folder: env::var("KRKNC_TEXTFILE_REMOVE_ALL_FOLDER_AFTER_READ").unwrap_or("false".to_string()).parse::<bool>().unwrap(),
            },
            camera: CameraCfg {
                enable: camera_enable,
                capture_interval_sec: env::var("KRKNC_CAMERA_CAPTURE_INTERVAL_SEC").unwrap_or("5".to_string()).parse::<u64>().unwrap(),
            },
            email: EmailCfg::default(),
            bjig: BjigCfg {
                enable: bjig_enable,
                device_path: env::var("KRKNC_BJIG_DEVICE_PATH").unwrap_or("/dev/ttyACM0".to_string()),
                cli_bin_path: env::var("KRKNC_BJIG_CLI_BIN_PATH").unwrap_or("./bin/bjig".to_string()),
                data_timeout_sec: env::var("KRKNC_BJIG_DATA_TIMEOUT_SEC").unwrap_or("300".to_string()).parse::<u64>().unwrap(),
                action_cooldown_sec: env::var("KRKNC_BJIG_ACTION_COOLDOWN_SEC").unwrap_or("30".to_string()).parse::<u64>().unwrap(),
            },
        }
    }
}
