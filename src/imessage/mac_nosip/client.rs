use anyhow::Result;

use crate::config::PlatformConfig;

pub struct MacNosipClient;

impl MacNosipClient {
    pub fn new(_config: &PlatformConfig) -> Result<Self> {
        Ok(Self)
    }
}
