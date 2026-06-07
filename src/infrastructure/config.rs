use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub theme: ThemeConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    pub default_theme: String,
    pub themes_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}








impl Config {
    pub fn from_file_and_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let content = fs::read_to_string("config.toml")?;
        let mut config: Config = toml::from_str(&content)?;
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }
        Ok(config)
    }
}