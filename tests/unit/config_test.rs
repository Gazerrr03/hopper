use hopper::core::config::{Config, Tool};
use tempfile::TempDir;

#[test]
fn test_default_config_has_tools() {
    let config = Config::default();
    assert_eq!(config.tools.len(), 2);
    assert!(config.find_tool("claude").is_some());
    assert!(config.find_tool("codex").is_some());
}

#[test]
fn test_add_tool() {
    let mut config = Config::default();
    let initial_len = config.tools.len();

    config.add_tool("neovim".to_string(), "nvim".to_string());

    assert_eq!(config.tools.len(), initial_len + 1);
    assert!(config.find_tool("neovim").is_some());
}

#[test]
fn test_find_tool_case_sensitive() {
    let config = Config::default();

    assert!(config.find_tool("claude").is_some());
    assert!(config.find_tool("Claude").is_none());
    assert!(config.find_tool("nonexistent").is_none());
}

#[test]
fn test_increment_tool_usage() {
    let mut config = Config::default();
    let tool = config.find_tool("claude").unwrap();
    assert_eq!(tool.recent, 0);

    config.increment_tool_usage("claude");
    let tool = config.find_tool("claude").unwrap();
    assert_eq!(tool.recent, 1);
}

#[test]
fn test_config_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("config.json");

    let mut config = Config::default();
    config.add_tool("test".to_string(), "test command".to_string());

    config.save_to_path(&path).unwrap();

    let loaded = Config::load_from_path(&path).unwrap();
    assert_eq!(loaded.tools.len(), config.tools.len());
}
