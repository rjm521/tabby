use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub openai: OpenAIConfig,
    pub api_desc: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let config = std::fs::read_to_string("config.toml").unwrap_or_else(|_| r#"api_desc = "默认配置" "#.to_string());
        toml::from_str(&config).unwrap_or_else(|_| AppConfig { api_desc: "默认配置".to_string(), openai: OpenAIConfig { api_key: String::new() } })
    }
}