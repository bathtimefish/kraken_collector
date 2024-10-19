use crate::{
    collectors::{
        CollectorFactory,
        webhook::WebhookFactory,
        mqtt::MqttFactory,
        websocket::WebsocketFactory,
        ibeacon::IbeaconFactory,
    },
    config::CollectorCfg
};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
pub async fn start(config: &CollectorCfg) -> Result<(), anyhow::Error> {
    let factories: Vec<Box<dyn CollectorFactory>> = vec![
        Box::new(WebhookFactory::new(config.clone())),
        Box::new(MqttFactory::new(config.clone())),
        Box::new(WebsocketFactory::new(config.clone())),
        Box::new(IbeaconFactory::new(config.clone())),
    ];

    let mut handles = Vec::new();

    for factory in &factories {
        let service = factory.create();
        let name = service.name();
        if service.is_enable() {
            debug!("starting {} collector service...", name);
            let handle = std::thread::spawn(move || {
                let started = service.start();
                match started {
                Ok(_) => debug!("{} collector started.", name),
                Err(e) => error!("Failed to start {} collector: {}", name, e),
                }
            });
            handles.push(handle);
        }
    }
    if handles.len() > 0 {
        debug!("collector service started.");
        for handle in handles {
            handle.join().unwrap();
        }
    } else {
        return Err(anyhow::anyhow!("all collector service are not enabled."));
    }
    Ok(())
}