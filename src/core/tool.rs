use crate::core::config::Tool;
use crate::error::{Result, ToolError};
use std::path::Path;
use std::process::Command;

pub fn replace_variables(command: &str, project_path: &Path) -> String {
    let path_str = project_path.to_string_lossy();
    let name_str = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    command
        .replace("$PROJECT_PATH", &path_str)
        .replace("$PROJECT_NAME", &name_str)
}

pub fn launch_tool(tool: &Tool, project_path: &Path, dry_run: bool) -> Result<()> {
    let command = replace_variables(&tool.command, project_path);

    if dry_run {
        println!("[Dry-run] Would execute: {}", command);
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", &command])
            .status()
            .map_err(|e| ToolError::LaunchFailed(e.to_string()))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("zsh")
            .arg("-i")
            .arg("-c")
            .arg(&command)
            .status()
            .map_err(|e| ToolError::LaunchFailed(e.to_string()))?;
    }

    Ok(())
}

pub fn open_shell(project_path: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[Dry-run] Would open shell in: {}", project_path.display());
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_string());

    #[cfg(not(target_os = "windows"))]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());

    Command::new(shell)
        .current_dir(project_path)
        .status()
        .map_err(|e| ToolError::LaunchFailed(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_replace_variables() {
        let path = PathBuf::from("/Users/qizhi/Projects/my-project");
        let cmd = "claude --path $PROJECT_PATH --name $PROJECT_NAME";
        let result = replace_variables(cmd, &path);
        assert!(result.contains("/Users/qizhi/Projects/my-project"));
        assert!(result.contains("my-project"));
    }

    #[test]
    fn test_replace_variables_no_vars() {
        let path = PathBuf::from("/test/project");
        let cmd = "echo hello";
        let result = replace_variables(cmd, &path);
        assert_eq!(result, "echo hello");
    }
}
