// direct4b Message Collector - Stub Implementation
//
// This is a stub implementation for the public repository.
// The full direct4b message collector implementation is available as a paid feature.
//
// Contact the maintainer for commercial licensing and access.

use super::{Collector, CollectorFactory};
use crate::config::CollectorCfg;

/// direct4b Message Collector (Stub)
///
/// This stub implementation is included in the public repository to maintain
/// code structure and allow compilation without the proprietary implementation.
///
/// The full implementation includes:
/// - Scheduled message collection with a timezone aware cron expression
/// - Period based collection (1hour / 6hour / 12hour / 1day / 3day / 1week / 1month)
/// - Paged retrieval that respects the direct4b message list API rate limit
/// - Message normalization and batched delivery to the Kraken Broker
///
/// Contact the maintainer for commercial licensing and access to the full implementation.
pub struct Direct4bCollector {
    #[allow(dead_code)]
    config: CollectorCfg,
}

pub struct Direct4bFactory {
    config: CollectorCfg,
}

impl Direct4bFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for Direct4bFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Direct4bCollector {
            config: self.config.clone(),
        })
    }
}

impl Collector for Direct4bCollector {
    fn name(&self) -> &'static str {
        "direct4b"
    }

    fn is_enable(&self) -> bool {
        // Always disabled in stub implementation
        false
    }

    fn start(&self) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!(
            "direct4b message collector is not available in the public version. \
             Please contact the maintainer for commercial licensing."
        ))
    }
}
