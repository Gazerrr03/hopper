use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub recent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "projectSets", default)]
    pub project_sets: Vec<PathBuf>,
    #[serde(default)]
    pub tools: Vec<Tool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_sets: Vec::new(),
            tools: vec![
                Tool {
                    name: "claude code".to_string(),
                    command: "claude".to_string(),
                    recent: 0,
                },
                Tool {
                    name: "codex".to_string(),
                    command: "codex".to_string(),
                    recent: 0,
                },
            ],
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        let base = config_dir().expect("Cannot find config directory");
        base.join("hopper").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).expect("Failed to read config");
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let content = serde_json::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(&path, content).expect("Failed to write config");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.project_sets.len(), 0);
        assert_eq!(config.tools.len(), 2);
    }
}
