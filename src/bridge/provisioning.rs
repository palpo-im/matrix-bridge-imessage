use anyhow::Result;

pub struct ProvisioningCoordinator;

impl ProvisioningCoordinator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProvisioningCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
