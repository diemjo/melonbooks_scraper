use crate::model::{Product};
use crate::common::error::Result;

pub mod melonbooks_scraper;

pub trait WebScraper {
    fn get_site_name(&self) -> &'static str;
    fn get_urls(&self, artist: &str, also_unavailable: bool) -> Result<Vec<String>>;
    fn get_product(&self, artist: &str, url: &str) -> Result<Product>;
}