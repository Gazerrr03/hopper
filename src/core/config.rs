use crate::error::{ConfigError, Result};
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
                    name: "claude".to_string(),
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
    fn default_config_dir() -> PathBuf {
        config_dir()
            .map(|p| p.join("hopper"))
            .unwrap_or_else(|| PathBuf::from(".hopper"))
    }

    fn default_config_path() -> PathBuf {
        Self::default_config_dir().join("config.json")
    }

    pub fn load_with_override(cli_config: Option<PathBuf>) -> Result<Self> {
        let path = cli_config
            .or_else(|| std::env::var("HOPPER_CONFIG").ok().map(PathBuf::from))
            .unwrap_or_else(Config::default_config_path);

        Self::load_from_path(&path)
    }

    fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()).into())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_config_path();
        self.save_to_path(&path)
    }

    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::WriteError(e.to_string()))?;
        }

        let content =
            serde_json::to_string_pretty(self).map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        Ok(())
    }

    pub fn add_tool(&mut self, name: String, command: String) {
        self.tools.push(Tool {
            name,
            command,
            recent: 0,
        });
    }

    pub fn find_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.name == name)
    }

    pub fn find_tool_mut(&mut self, name: &str) -> Option<&mut Tool> {
        self.tools.iter_mut().find(|t| t.name == name)
    }

    pub fn increment_tool_usage(&mut self, name: &str) {
        if let Some(tool) = self.find_tool_mut(name) {
            tool.recent += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.project_sets.is_empty());
        assert_eq!(config.tools.len(), 2);
    }

    #[test]
    fn test_add_tool() {
        let mut config = Config::default();
        config.add_tool("test".to_string(), "test command".to_string());
        assert_eq!(config.tools.len(), 3);
    }

    #[test]
    fn test_find_tool() {
        let config = Config::default();
        assert!(config.find_tool("claude").is_some());
        assert!(config.find_tool("nonexistent").is_none());
    }
}
