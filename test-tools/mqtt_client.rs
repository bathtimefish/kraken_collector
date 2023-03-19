use tokio::{task, time};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use serde::{Serialize, Deserialize};
extern crate log;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: i32,
    message: String,
    status: bool,
}

#[tokio::main(worker_threads = 1)]
async fn main() {
    env_logger::init();
    let mut mqttoptions = MqttOptions::new("kraken-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    task::spawn(async move {
        requests(client).await;
        time::sleep(Duration::from_secs(3)).await;
    });

    loop {
        let event = eventloop.poll().await;
        match &event {
            Ok(e) => {
                println!("Event: {:?}", e);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

async fn requests(client: AsyncClient) {
    client.subscribe("kraken", QoS::AtMostOnce).await.unwrap();
    let mut status = true;
    for i in 0..10_usize {
        let message = Message {
            id: i.try_into().unwrap(),
            message: format!("hello-kraken-{}", i),
            status,
        };
        let payload = serde_json::to_string(&message).unwrap().as_bytes().to_vec();
        let topic = format!("kraken");
        let qos = QoS::AtMostOnce;
        client.publish(topic, qos, false, payload).await.unwrap();
        status = !status;
        time::sleep(Duration::from_secs(1)).await;
    }
    time::sleep(Duration::from_secs(120)).await;
}
