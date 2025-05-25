use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:2794").await?;
    println!("WebSocket test server listening on ws://127.0.0.1:2794");

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New connection from {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("Error accepting WebSocket connection from {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send initial hello message
    if let Err(e) = ws_sender.send(Message::Text("Hello".to_string().into())).await {
        println!("Error sending hello message to {}: {}", addr, e);
        return;
    }

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text from {}: {}", addr, text);
                // Echo the message back
                if let Err(e) = ws_sender.send(Message::Text(format!("Echo: {}", text).into())).await {
                    println!("Error sending echo to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Binary(data)) => {
                println!("Received binary from {} ({} bytes)", addr, data.len());
                // Echo the binary data back
                if let Err(e) = ws_sender.send(Message::Binary(data)).await {
                    println!("Error sending binary echo to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Ping(ping_data)) => {
                println!("Received ping from {}", addr);
                if let Err(e) = ws_sender.send(Message::Pong(ping_data)).await {
                    println!("Error sending pong to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                println!("Received pong from {}", addr);
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed by client {}", addr);
                let _ = ws_sender.send(Message::Close(None)).await;
                break;
            }
            Err(e) => {
                println!("Error receiving message from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }

    println!("Connection with {} ended", addr);
}
