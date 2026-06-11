use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub memory_db_path: PathBuf,
    pub llm_endpoint: String,
    pub llm_model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            memory_db_path: home.join(".nakama_config").join("memory.db"),
            llm_endpoint: "".to_string(),
            llm_model: "".to_string(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&app_data_dir);
        ConfigManager { config_path: app_data_dir.join("app_config.json") }
    }

    pub fn load(&self) -> Result<AppConfig, String> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)
                .map_err(|e| format!("failed to read config: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("failed to parse config: {}", e))
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("failed to serialize config: {}", e))?;
        fs::write(&self.config_path, content)
            .map_err(|e| format!("failed to write config: {}", e))?;
        Ok(())
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
