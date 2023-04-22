#[macro_use]
extern crate log;
use anyhow::Result;
mod service;
mod collectors;
mod config;
use crate::config::CollectorCfg;

fn main() -> Result<(), anyhow::Error>{
    let config = CollectorCfg::default();
    env_logger::init();
    info!("KRAKEN Collector -- The Highlevel Data Collector -- boot squence start.");
    service::start(&config).unwrap_or_else(|e| {
        error!("Failed to start collector service: {}", e)
    });
    Ok(())
}
