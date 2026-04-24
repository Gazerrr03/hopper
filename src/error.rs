use thiserror::Error;

#[derive(Error, Debug)]
pub enum HopperError {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Project error: {0}")]
    Project(#[from] ProjectError),

    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Cannot read config: {0}")]
    ReadError(String),

    #[error("Cannot parse config: {0}")]
    ParseError(String),

    #[error("Cannot write config: {0}")]
    WriteError(String),
}

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Project not found: {0}")]
    NotFound(String),

    #[error("Cannot delete project: {0}")]
    DeleteError(String),
}

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Tool launch failed: {0}")]
    LaunchFailed(String),
}

#[derive(Error, Debug)]
pub enum UiError {
    #[error("fzf is not installed. Run: brew install fzf")]
    FzfNotFound,

    #[error("{0}")]
    UnsupportedTerminal(String),

    #[error("fzf process error: {0}")]
    ProcessError(String),
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cannot access cache directory: {0}")]
    AccessError(String),

    #[error("Cannot read cache: {0}")]
    ReadError(String),

    #[error("Cannot write cache: {0}")]
    WriteError(String),
}

pub type Result<T> = std::result::Result<T, HopperError>;
