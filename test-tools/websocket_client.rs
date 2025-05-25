use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::io::{self as std_io, Write};

const CONNECTION: &str = "ws://127.0.0.1:2794";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to {}", CONNECTION);

    let (ws_stream, _) = connect_async(CONNECTION).await?;
    println!("Successfully connected");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Spawn task for receiving messages
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text: {}", text);
                }
                Ok(Message::Binary(data)) => {
                    println!("Received binary: {} bytes", data.len());
                }
                Ok(Message::Ping(data)) => {
                    println!("Received ping: {:?}", data);
                }
                Ok(Message::Pong(data)) => {
                    println!("Received pong: {:?}", data);
                }
                Ok(Message::Close(_)) => {
                    println!("Connection closed by server");
                    break;
                }
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle user input
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    println!("Enter messages (type '/close' to exit, '/ping' to send ping):");
    print!("> ");
    std_io::stdout().flush().unwrap();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                
                let message = match trimmed {
                    "/close" => {
                        let _ = ws_sender.send(Message::Close(None)).await;
                        break;
                    }
                    "/ping" => Message::Ping(b"PING".to_vec().into()),
                    _ => Message::Text(trimmed.to_string().into()),
                };

                if let Err(e) = ws_sender.send(message).await {
                    println!("Error sending message: {}", e);
                    break;
                }
                
                print!("> ");
                std_io::stdout().flush().unwrap();
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }

    // Wait for receive task to complete
    let _ = receive_task.await;
    println!("Exited");
    
    Ok(())
}
