use std::sync::Arc;

use anyhow::Result;

pub struct AdminHandler;

impl AdminHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AdminHandler {
    fn default() -> Self {
        Self::new()
    }
}
