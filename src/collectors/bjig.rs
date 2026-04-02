// BraveJIG Collector - Stub Implementation
//
// This is a stub implementation for the public repository.
// The full BraveJIG collector implementation is available as a paid feature.
//
// Contact the maintainer for commercial licensing and access.

use crate::config::CollectorCfg;
use super::{Collector, CollectorFactory};

/// BraveJIG Collector (Stub)
///
/// This stub implementation is included in the public repository to maintain
/// code structure and allow compilation without the proprietary bjig_controller library.
///
/// The full implementation includes:
/// - Automatic router connection and monitoring
/// - Real-time sensor data collection
/// - Bidirectional communication with Kraken Broker
/// - Automatic reconnection on timeout
/// - Action command processing with pause/resume
///
/// Contact the maintainer for commercial licensing and access to the full implementation.
pub struct Bjig {
    #[allow(dead_code)]
    config: CollectorCfg,
}

pub struct BjigFactory {
    config: CollectorCfg,
}

impl BjigFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for BjigFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Bjig { config: self.config.clone() })
    }
}

impl Collector for Bjig {
    fn name(&self) -> &'static str {
        "bjig"
    }

    fn is_enable(&self) -> bool {
        // Always disabled in stub implementation
        false
    }

    fn start(&self) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!(
            "BraveJIG collector is not available in the public version. \
             Please contact the maintainer for commercial licensing."
        ))
    }
}