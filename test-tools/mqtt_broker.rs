use rumqttd::{Broker, Config, Notification};
use std::thread;
use std::str;
#[macro_use]
extern crate log;

fn main() {
    env_logger::init();
    let config = config::Config::builder()
        .add_source(config::File::with_name("config/rumqttd.toml"))
        .build()
        .unwrap();
    let config: Config = config.try_deserialize().unwrap();
    /*
    let mut config: Config = Config::default();
    config.router.max_connections = 1000;
    config.router.max_segment_size = 104857600;
    config.router.max_read_len = 102400;
    config.router.max_segment_count = 10;
    */

    let mut broker = Broker::new(config);
    let (mut tx, mut rx) = broker.link("kraken").unwrap();
    thread::spawn(move || {
        broker.start().unwrap();
    });
    tx.subscribe("kraken").unwrap();
    let mut count = 0;
    loop {
        let notification = match rx.recv().unwrap() {
            Some(notification) => {
                println!("!!!! {:?}", notification);
                notification
            }
            None => continue,
        };
        warn!("{}: Notification: {:?}", count, notification);
        match notification {
            // !!! メッセージが受け取れない..
            Notification::Forward(forward) => {
                count += 1;
                info!("{}: Forward: {:?}", count, forward);
                let bytes: &[u8] = &forward.publish.payload;
                let message = str::from_utf8(bytes).unwrap();
                warn!("{}: Message: {:?}", count, message);
            }
            Notification::Disconnect(a, b) => {
                info!("ForwardWithProperties: {:?}, {:?}", a, b);
            }
            v => {
                warn!("{v:?}");
                continue;
            }
        };
    }
}