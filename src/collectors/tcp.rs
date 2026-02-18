use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;

#[derive(Debug, serde::Serialize)]
struct MetaData {
    peer_addr: String,
}

pub struct Tcp {
    config: CollectorCfg,
}

pub struct TcpFactory {
    config: CollectorCfg,
}

impl TcpFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for TcpFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Tcp { config: self.config.clone() })
    }
}

impl Collector for Tcp {
    fn name(&self) -> &'static str {
        "tcp"
    }

    fn is_enable(&self) -> bool {
        self.config.tcp.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let addr = format!("{}:{}", self.config.tcp.host, self.config.tcp.port);
        let grpc_config = self.config.grpc.clone();
        let buffer_size = self.config.tcp.buffer_size;

        let listener = TcpListener::bind(&addr).await?;
        info!("TCP collector listening on {} (buffer_size={})", addr, buffer_size);

        loop {
            match listener.accept().await {
                Ok((mut stream, peer_addr)) => {
                    let grpc_config = grpc_config.clone();
                    let peer_addr_str = peer_addr.to_string();
                    info!("TCP client connected: {}", peer_addr_str);

                    tokio::spawn(async move {
                        let mut buf = vec![0u8; buffer_size];
                        loop {
                            match stream.read(&mut buf).await {
                                Ok(0) => {
                                    info!("TCP client disconnected: {}", peer_addr_str);
                                    break;
                                }
                                Ok(n) => {
                                    debug!("Received {} bytes from {}", n, peer_addr_str);
                                    let metadata = MetaData {
                                        peer_addr: peer_addr_str.clone(),
                                    };
                                    let meta_json = json!(metadata);
                                    match grpc::send(
                                        &grpc_config,
                                        "tcp",
                                        "application/octet-stream",
                                        &serde_json::to_string(&meta_json).unwrap(),
                                        &buf[..n],
                                    )
                                    .await
                                    {
                                        Ok(response) => {
                                            debug!("Sent {} bytes from {} to gRPC", n, peer_addr_str);
                                            let kraken_response = response.into_inner();
                                            // response_type=tcp のとき、payloadをTCPクライアントに書き戻す
                                            if !kraken_response.payload.is_empty() {
                                                if let Ok(response_meta) = serde_json::from_str::<serde_json::Value>(&kraken_response.metadata) {
                                                    if response_meta.get("response_type").and_then(|v| v.as_str()) == Some("tcp") {
                                                        match stream.write_all(&kraken_response.payload).await {
                                                            Ok(_) => debug!("Sent {} bytes response to TCP client {}", kraken_response.payload.len(), peer_addr_str),
                                                            Err(e) => error!("Failed to write response to TCP client {}: {:?}", peer_addr_str, e),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => error!("Failed to send to gRPC: {:?}", e),
                                    }
                                }
                                Err(e) => {
                                    error!("TCP read error from {}: {:?}", peer_addr_str, e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("TCP accept error: {:?}", e);
                }
            }
        }
    }
}
