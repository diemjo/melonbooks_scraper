use directories::BaseDirs;
use figment::Figment;
use figment::providers::{Format, Serialized, Yaml};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AppConfig {
    pub mysql_username: String,
    pub mysql_password: String,
    pub mysql_host: String,
    pub mysql_port: u32,
    pub mysql_db: String,
}

impl AppConfig {

    pub fn load_config() -> Self {
        Figment::from(Serialized::defaults(AppConfig::default()))
            .merge(Yaml::file("/etc/melonbooks_scraper/config.yaml"))
            .merge(Yaml::file(BaseDirs::new().unwrap().config_dir().join("melonbooks_scraper.yaml").as_path()))
            .merge(Yaml::file("./melonbooks_scraper.yaml"))
            .extract()
            .unwrap()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            mysql_username: "melonbooks".to_string(),
            mysql_password: "password".to_string(),
            mysql_host: "localhost".to_string(),
            mysql_port: 3306,
            mysql_db: "melonbooks".to_string()
        }
    }
}