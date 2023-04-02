extern crate websocket;
use websocket::sync::Server;
use websocket::OwnedMessage;

use crate::config::CollectorConfig;

use super::Collector;
use super::grpc;

pub struct Websocket {
    pub config: CollectorConfig,
}

const SUB_PROTOCOL: &'static str = "kraken-ws";

impl Collector for Websocket {
    fn name(&self) -> &str {
        "websocket"
    }
    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let host = self.config.websocket.host.clone();
        // let sub_protocol = self.config.websocket.sub_protocol.clone(); // <-- うまくいかない
        let server = Server::bind(host.clone()).unwrap();
        debug!("Websocket server was started that is listening on ws://{}", &host);

        for request in server.filter_map(Result::ok) {
            // Spawn a new thread for each connection.
            // !!!! thread::spawn(move || async {  <- asyncを入れると動作しない
            std::thread::spawn(move || {
                let sub_protocol = SUB_PROTOCOL;
                let mut client = request.use_protocol(sub_protocol).accept().unwrap();
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
                        OwnedMessage::Text(text) => {
                            debug!("Received text: {}", text);
                            tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap()
                                .block_on(grpc::send(&text, "websocket")).unwrap();
                        }
                        _ => sender.send_message(&message).unwrap(),
                    }
                }
            });
        }
        Ok(())
    }
}
