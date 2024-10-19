use std::error::Error;
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CentralEvent};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;

#[derive(Debug, serde::Serialize)]
struct IBeaconData {
    uuid: String,
    rssi: i16,
    address: String,
    major: u16,
    minor: u16,
}

#[derive(Debug, Deserialize)]
struct Config {
    allowed_uuids: Vec<String>,
}

pub struct Ibeacon {
    config: CollectorCfg,
}

pub struct IbeaconFactory {
    config: CollectorCfg,
}

impl IbeaconFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for IbeaconFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Ibeacon { config: self.config.clone() })
    }
}

impl Collector for Ibeacon {
    fn name(&self) -> &'static str {
        "ibeacon"
    }

    fn is_enable(&self) -> bool {
        self.config.ibeacon.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let grpc_config = self.config.grpc.clone();
        debug!("Using allowed uuid list: {}", &self.config.ibeacon.allowed_uuid_filter_path);
        let config = match load_config(&self.config.ibeacon.allowed_uuid_filter_path) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load configuration: {}", e);
                return Err(e);
            }
        };

        let allowed_uuids: Vec<Uuid> = config
            .allowed_uuids
            .into_iter()
            .filter_map(|uuid_str| Uuid::parse_str(&uuid_str).ok())
            .collect();

        if allowed_uuids.is_empty() {
            debug!("No UUID filtering is applied, all iBeacons will be processed.");
        } else {
            debug!("Loaded allowed UUIDs: {:?}", allowed_uuids);
        }

        let manager = Manager::new().await?;
        let adapter_list = manager.adapters().await?;
        debug!("Adapter list obtained: {:?}", adapter_list);

        if adapter_list.is_empty() {
            error!("No Bluetooth adapters found");
            return Ok(());
        }

        let adapter = adapter_list.into_iter().nth(0).unwrap();
        debug!("Adapter selected: {}", adapter.adapter_info().await?);

        // set filter duration 
        let filter_duration_secs = self.config.ibeacon.filter_duration; 
        let filter_duration = Duration::from_secs(filter_duration_secs);

        debug!("Using filter interval duration: {} seconds", filter_duration_secs);

        // start scaning 
        let scan_filter = ScanFilter {
            services: vec![],  // scan all services 
        };
        adapter.start_scan(scan_filter).await?;

        let seen_ibeacons = Arc::new(Mutex::new(HashMap::new()));
        let mut events = adapter.events().await?;
        loop {
            tokio::select! {
                Some(event) = events.next() => {
                    match event {
                        CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } => {
                            if let Some(data) = manufacturer_data.get(&0x004C) { // Company Identifier of Apple
                                let peripheral = adapter.peripheral(&id).await?;
                                let seen_ibeacons = seen_ibeacons.clone();
                                let data = data.clone();
                                let allowed_uuids = allowed_uuids.clone();
                                let filter_duration = filter_duration.clone();
                                tokio::spawn({
                                    let grpc_config = grpc_config.clone();
                                    async move {
                                        if let Err(e) = process_ibeacon_data(
                                            &peripheral,
                                            &data,
                                            seen_ibeacons,
                                            filter_duration,
                                            allowed_uuids,
                                            &grpc_config)
                                        .await {
                                            error!("Error processing iBeacon data: {}", e);
                                        }
                                    }
                                });
                            }
                        },
                        _ => {}
                    }
                },
            }
        }
        //adapter.stop_scan().await?;
        //debug!("Scan stopped. Exiting...");
    }
}

async fn process_ibeacon_data(
    peripheral: &impl Peripheral,
    data: &[u8],
    seen_ibeacons: Arc<Mutex<HashMap<String, Instant>>>,
    filter_duration: Duration,
    allowed_uuids: Vec<Uuid>,
    grpc_config: &crate::config::GrpcCfg,
) -> Result<(), Box<dyn Error>> {
    if data.len() >= 23 && data[0] == 0x02 && data[1] == 0x15 {
        let uuid = Uuid::from_slice(&data[2..18])?;

        // check if the UUID is allowed
        if !allowed_uuids.is_empty() && !allowed_uuids.contains(&uuid) {
            return Ok(()); // omit this iBeacon which UUID is not allowed or list was empty
        }

        let major = u16::from_be_bytes([data[18], data[19]]);
        let minor = u16::from_be_bytes([data[20], data[21]]);
        let _tx_power = data[22] as i8;

        let address = peripheral.address().to_string();
        let properties = peripheral.properties().await?.ok_or("No properties")?;
        let rssi = properties.rssi.unwrap_or(0);
        let local_name = properties.local_name.unwrap_or_else(|| String::from("Unknown"));

        // create a key for the iBeacon 
        let ibeacon_key = format!("{}:{}:{}", uuid, major, minor);

        // get the current time 
        let now = Instant::now();

        {
            let mut seen = seen_ibeacons.lock().unwrap();
            // get the last seen time
            if let Some(&last_seen) = seen.get(&ibeacon_key) {
                // skip if the last seen time is within the filter duration
                if now.duration_since(last_seen) < filter_duration {
                    return Ok(());
                }
            }
            // set the last seen time
            seen.insert(ibeacon_key.clone(), now);
        }

        let ibeacon_data = IBeaconData {
            uuid: uuid.to_string(),
            rssi,
            address: address.clone(),
            major,
            minor,
        };

        let json = json!(ibeacon_data);
        debug!("iBeacon detected: {} ({}), UUID: {}, Major: {}, Minor: {}, RSSI: {}",
              local_name, address, uuid, major, minor, rssi);
        debug!("JSON: {}", serde_json::to_string_pretty(&json)?);
        let sent = grpc::send(&grpc_config, &serde_json::to_string(&json).unwrap(), &"ibeacon").await;
    
        match sent {
            Ok(msg) => debug!("Sent message to grpc server: {:?}", msg),
            Err(msg) => error!("Failed to send to grpc: {:?}", msg),
        }
    }
    Ok(())
}

// load configuration from the yaml file
fn load_config(path: &str) -> Result<Config, anyhow::Error> {
    match fs::read_to_string(path) {
        Ok(content) => {
            match serde_yaml::from_str::<Config>(&content) {
                Ok(config) => Ok(config),
                Err(e) => {
                    error!("Failed to parse YAML file: {}", e);
                    Err(anyhow::Error::new(e))
                }
            }
        }
        Err(e) => {
            error!("Failed to read YAML file at {}: {}", path, e);
            Err(anyhow::Error::new(e))
        }
    }
}