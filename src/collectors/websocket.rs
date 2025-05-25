use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;

use crate::config::CollectorCfg;

use super::Collector;
use super::CollectorFactory;
use super::grpc;

#[derive(Debug, Clone)]
pub struct Websocket {
    pub config: CollectorCfg,
}

pub struct WebsocketFactory {
    pub config: CollectorCfg,
}

impl WebsocketFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for WebsocketFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Websocket{ config: self.config.clone() })
    }
}

impl Collector for Websocket {
    fn name(&self) -> &'static str {
        "websocket"
    }
    
    fn is_enable(&self) -> bool {
       self.config.websocket.enable
    }
    
    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let ws_config = self.config.websocket.clone();
        let grpc_config = self.config.grpc.clone();
        
        let listener = TcpListener::bind(&ws_config.host).await?;
        debug!("WebSocket server started, listening on ws://{}", &ws_config.host);

        while let Ok((stream, addr)) = listener.accept().await {
            let grpc_config = grpc_config.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, addr, grpc_config).await {
                    error!("Error handling WebSocket connection from {}: {}", addr, e);
                }
            });
        }
        
        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    grpc_config: crate::config::GrpcCfg,
) -> Result<(), anyhow::Error> {
    debug!("New WebSocket connection from {}", addr);
    
    let ws_stream = accept_async(stream).await?;
    debug!("WebSocket connection established with {}", addr);
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send initial hello message
    let hello_message = Message::Text("{\"kraken\": \"hello\"}".to_string().into());
    ws_sender.send(hello_message).await?;
    
    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                debug!("Received text message from {}: {}", addr, text);
                
                // Send to gRPC server
                let sent = grpc::send(
                    &grpc_config,
                    "websocket",
                    "application/json",
                    "{}",
                    text.as_bytes()
                ).await;
                
                if let Err(e) = sent {
                    error!("Failed to send message to gRPC server: {}", e);
                    continue;
                }
                
                let response = sent.unwrap();
                let kraken_response = response.into_inner();
                
                // Check if response should be sent back to this WebSocket client
                if kraken_response.collector_name == "websocket" {
                    let response_message = match kraken_response.content_type.as_str() {
                        "application/json" | "text/plain" | "text/html" => {
                            // Send as text message
                            let text = String::from_utf8_lossy(&kraken_response.payload);
                            Message::Text(text.to_string().into())
                        }
                        _ => {
                            // Send as binary message for other content types
                            Message::Binary(kraken_response.payload.into())
                        }
                    };
                    
                    if let Err(e) = ws_sender.send(response_message).await {
                        error!("Failed to send response to WebSocket client {}: {}", addr, e);
                        break;
                    }
                }
            }
            Message::Binary(data) => {
                debug!("Received binary message from {} ({} bytes)", addr, data.len());
                
                // Send to gRPC server
                let sent = grpc::send(
                    &grpc_config,
                    "websocket",
                    "application/octet-stream",
                    "{}",
                    &data
                ).await;
                
                if let Err(e) = sent {
                    error!("Failed to send message to gRPC server: {}", e);
                    continue;
                }
                
                let response = sent.unwrap();
                let kraken_response = response.into_inner();
                
                // Check if response should be sent back to this WebSocket client
                if kraken_response.collector_name == "websocket" {
                    let response_message = match kraken_response.content_type.as_str() {
                        "application/json" | "text/plain" | "text/html" => {
                            // Send as text message
                            let text = String::from_utf8_lossy(&kraken_response.payload);
                            Message::Text(text.to_string().into())
                        }
                        _ => {
                            // Send as binary message for other content types
                            Message::Binary(kraken_response.payload.into())
                        }
                    };
                    
                    if let Err(e) = ws_sender.send(response_message).await {
                        error!("Failed to send response to WebSocket client {}: {}", addr, e);
                        break;
                    }
                }
            }
            Message::Ping(ping_data) => {
                debug!("Received ping from {}", addr);
                let pong_message = Message::Pong(ping_data);
                if let Err(e) = ws_sender.send(pong_message).await {
                    error!("Failed to send pong to {}: {}", addr, e);
                    break;
                }
            }
            Message::Pong(_) => {
                debug!("Received pong from {}", addr);
                // Pong messages are typically just acknowledged
            }
            Message::Close(_) => {
                debug!("WebSocket connection closed by client {}", addr);
                break;
            }
            Message::Frame(_) => {
                // Raw frames are typically not handled directly
                debug!("Received raw frame from {}", addr);
            }
        }
    }
    
    debug!("WebSocket connection with {} ended", addr);
    Ok(())
}
