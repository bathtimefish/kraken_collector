#[macro_use]
extern crate log;
use std::env;
use anyhow::Result;

mod service;
pub mod collectors;
pub mod config;
use crate::config::CollectorCfg;

fn main() -> Result<(), anyhow::Error>{
    let config = CollectorCfg::default();
    let log_level = env::var("LOG_LEVEL").unwrap_or("info".to_string());
    env_logger::init();
    info!("RUST_ROG: {}", log_level);
    info!("KRAKEN Collector -- The Highlevel Data Collector -- boot squence start.");
    service::start(&config).unwrap();
    Ok(())
}
