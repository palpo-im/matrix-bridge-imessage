use anyhow::Result;

pub struct MediaHandler;

impl MediaHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn process(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement media processing
        Ok(Vec::new())
    }
}

impl Default for MediaHandler {
    fn default() -> Self {
        Self::new()
    }
}
