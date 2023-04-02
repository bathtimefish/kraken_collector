use crate::{
    collectors::{
        CollectorFactory,
        webhook::WebhookFactory,
        mqtt::MqttFactory,
        websocket::WebsocketFactory, 
    },
    config::CollectorConfig
};

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
pub async fn start(config: CollectorConfig) -> Result<(), anyhow::Error> {
    let factories: Vec<Box<dyn CollectorFactory>> = vec![
        Box::new(WebhookFactory::new(config.clone())),
        Box::new(MqttFactory::new(config.clone())),
        Box::new(WebsocketFactory::new(config.clone())),
    ];

    let mut handles = Vec::new();

    for factory in &factories {
        let service = factory.create();
        let name = service.name();
        let handle = std::thread::spawn(move || {
            let started = service.start();
            match started {
               Ok(_) => debug!("{} collector started.", name),
               Err(e) => error!("Failed to start {} collector: {}", name, e),
            }
        });
        handles.push(handle);
    }

    debug!("collector service started.");
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}