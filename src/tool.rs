use crate::config::Tool;
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

pub fn launch_tool(tool: &Tool, project_path: &Path) {
    let command = replace_variables(&tool.command, project_path);

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", &command])
            .spawn()
            .expect("Failed to launch tool");
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("zsh")
            .arg("-c")
            .arg(&command)
            .spawn()
            .expect("Failed to launch tool");
    }
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
}
