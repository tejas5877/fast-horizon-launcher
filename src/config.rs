use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "launcher-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub player_name: String,
    pub minecraft_dir: Option<PathBuf>,
    pub java_home: Option<PathBuf>,
    pub offline_mode: bool,
    pub horizon_data_dir: Option<PathBuf>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            player_name: "Player".to_string(),
            minecraft_dir: None,
            java_home: None,
            offline_mode: true,
            horizon_data_dir: None,
        }
    }
}

impl LauncherConfig {
    pub fn config_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(CONFIG_FILE)))
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE))
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            let default_cfg = Self::default();
            let _ = default_cfg.save();
            return default_cfg;
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    pub fn set_player_name(&mut self, name: &str) -> std::io::Result<()> {
        let sanitized = name.trim();
        if !sanitized.is_empty() {
            self.player_name = sanitized.to_string();
            self.save()?;
        }
        Ok(())
    }

    pub fn set_minecraft_dir<P: AsRef<Path>>(&mut self, path: Option<P>) -> std::io::Result<()> {
        self.minecraft_dir = path.map(|p| p.as_ref().to_path_buf());
        self.save()
    }

    pub fn set_java_home<P: AsRef<Path>>(&mut self, path: Option<P>) -> std::io::Result<()> {
        self.java_home = path.map(|p| p.as_ref().to_path_buf());
        self.save()
    }
}
