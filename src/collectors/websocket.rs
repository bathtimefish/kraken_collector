extern crate websocket;
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;

use super::Collector;
use super::grpc;

pub(crate) struct Websocket;

const HOST_ADDR: &'static str = "127.0.0.1:2794";
const SUB_PROTOCOL: &'static str = "kraken-ws";

impl Collector for Websocket {
    fn new() -> Self {
        Websocket {}
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let server = Server::bind(HOST_ADDR).unwrap();

        for request in server.filter_map(Result::ok) {
            // Spawn a new thread for each connection.
            // !!!! thread::spawn(move || async {  <- asyncを入れると動作しない
            thread::spawn(move || {
                let mut client = request.use_protocol(SUB_PROTOCOL).accept().unwrap();
                let ip = client.peer_addr().unwrap();
                println!("Connection from {}", ip);
                let message = OwnedMessage::Text("{\"kraken\": \"hello\"}".to_string());
                client.send_message(&message).unwrap();
                let (mut receiver, mut sender) = client.split().unwrap();
                for message in receiver.incoming_messages() {
                    let message = message.unwrap();

                    match message {
                        OwnedMessage::Close(_) => {
                            let message = OwnedMessage::Close(None);
                            sender.send_message(&message).unwrap();
                            println!("Client {} disconnected", ip);
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
