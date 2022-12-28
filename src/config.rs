use std::path::{PathBuf};
use figment::Figment;
use figment::providers::{Format, Serialized, Yaml};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AppConfig {
    pub db_path: PathBuf,
}

impl AppConfig {

    pub fn load_config() -> Self {
        Figment::from(Serialized::defaults(AppConfig::default()))
            .merge(Yaml::file("/config/melonbooks_scraper.yaml"))
            .merge(Yaml::file("./config/melonbooks_scraper.yaml"))
            .merge(Yaml::file("./melonbooks_scraper.yaml"))
            .extract()
            .unwrap()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            db_path: PathBuf::from("/data/melonbooks.db"),
        }
    }
}