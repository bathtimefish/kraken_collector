extern crate websocket;
use websocket::sync::Server;
use websocket::OwnedMessage;

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
        let server = Server::bind(&ws_config.host).unwrap();
        debug!("Websocket server was started that is listening on ws://{}", &ws_config.host);

        for request in server.filter_map(Result::ok) {
            // Spawn a new thread for each connection.
            std::thread::spawn({
                let sub_protocol = ws_config.sub_protocol.clone();
                let grpc_config = self.config.grpc.clone();
                move || {
                    // let mut client = request.accept().unwrap();
                    let mut client = request.use_protocol(&sub_protocol).accept().unwrap();
                    let ip = client.peer_addr().unwrap();
                    debug!("Connection from {}", ip);
                    let message = OwnedMessage::Text("{\"kraken\": \"hello\"}".to_string());
                    client.send_message(&message).unwrap();
                    let (mut receiver, mut sender) = client.split().unwrap();
                    for message in receiver.incoming_messages() {
                        let message = message.unwrap_or_else(|e| {
                            match e {
                                websocket::WebSocketError::NoDataAvailable => {
                                    OwnedMessage::Close(None)
                                }
                                _ => {
                                    debug!("Error: {:?}", e);
                                    OwnedMessage::Close(None)
                                }
                            }
                        });

                        match message {
                            OwnedMessage::Close(_) => {
                                let message = OwnedMessage::Close(None);
                                sender.send_message(&message).unwrap();
                                debug!("Client {} disconnected", ip);
                                return;
                            }
                            OwnedMessage::Ping(ping) => {
                                let message = OwnedMessage::Pong(ping);
                                sender.send_message(&message).unwrap();
                            }
                            OwnedMessage::Text(payload) => {
                                debug!("Received text: {}", payload);
                                tokio::runtime::Builder::new_current_thread()
                                    .enable_all()
                                    .build()
                                    .unwrap()
                                    .block_on(grpc::send(&grpc_config, &payload, "websocket")).unwrap();
                            }
                            _ => sender.send_message(&message).unwrap(),
                        }
                    }
                }
            });
        }
        Ok(())
    }
}
