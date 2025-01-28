use std::io::{self};
use std::time::Duration;
use serde_json::json;
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;

#[derive(Debug, serde::Serialize)]
struct SerialData {
    hex_string: String,
}

pub struct Serial {
    config: CollectorCfg,
}

pub struct SerialFactory {
    config: CollectorCfg,
}

impl SerialFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for SerialFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Serial { config: self.config.clone() })
    }
}

impl Collector for Serial {
    fn name(&self) -> &'static str {
        "serial"
    }

    fn is_enable(&self) -> bool {
        self.config.serial.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let port_name = self.config.serial.port.clone();
        let baud_rate = self.config.serial.baudrate;
        let timeout_sec = self.config.serial.timeout;

        let port = serialport::new(&port_name, baud_rate)
            .timeout(Duration::from_millis(timeout_sec))
            .open();
        debug!("Connecting to serial device on {} at {} baud:", &port_name, &baud_rate);

        match port {
            Ok(mut port) => {
                let mut serial_buf: Vec<u8> = vec![0; 1024];
                loop {
                    match port.read(serial_buf.as_mut_slice()) {
                        Ok(t) => {
                            if t > 0 {
                                // encode to hex string 
                                let hex_string: String = serial_buf[..t]
                                    .iter()
                                    .map(|b| format!("{:02x}", b))
                                    .collect();
                                debug!("Received {} bytes: {}", t, hex_string);
                                let data = SerialData { hex_string };
                                let json = json!(data);
                                let sent = grpc::send(&self.config.grpc, &serde_json::to_string(&json).unwrap(), &"serial").await;
                                match sent {
                                    Ok(msg) => debug!("Sent message to grpc server: {:?}", msg),
                                    Err(msg) => error!("Failed to send to grpc: {:?}", msg),
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            Err(e) => {
                error!("Failed to open \"{}\". Error: {}", &port_name, e);
                //::std::process::exit(1);
            }
        }
        Ok(())
    }

}