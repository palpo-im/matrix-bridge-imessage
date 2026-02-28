use anyhow::Result;
use url::Url;

#[derive(Debug, Clone)]
pub struct UrlPreview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

impl UrlPreview {
    pub async fn fetch(url: &str) -> Result<Self> {
        // TODO: Implement URL preview fetching
        Ok(Self {
            url: url.to_string(),
            title: None,
            description: None,
            image_url: None,
        })
    }
}
