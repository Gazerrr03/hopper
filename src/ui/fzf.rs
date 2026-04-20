use crate::core::config::Tool;
use crate::core::project::Project;
use crate::error::{Result, UiError};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionType {
    Enter,
    Delete,
}

#[derive(Debug, Clone)]
pub struct ProjectSelection {
    pub index: usize,
    pub selection_type: SelectionType,
}

pub trait UiBackend: Send + Sync {
    fn project_selection(&self, projects: &[Project]) -> Result<Option<ProjectSelection>>;
    fn tool_selection(&self, tools: &[Tool]) -> Result<Option<usize>>;
    fn confirm_deletion(&self, name: &str) -> Result<bool>;
    fn add_tool_interactive(&self) -> Result<Option<(String, String)>>;
}

fn check_fzf() -> Result<()> {
    Command::new("fzf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| UiError::FzfNotFound)?;

    Ok(())
}

pub struct FzfBackend;

impl FzfBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FzfBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl UiBackend for FzfBackend {
    fn project_selection(&self, projects: &[Project]) -> Result<Option<ProjectSelection>> {
        check_fzf()?;

        let items: Vec<String> = projects
            .iter()
            .map(|p| format!("{}\t{}", p.display_name(), p.mtime_str()))
            .collect();

        if items.is_empty() {
            return Ok(None);
        }

        let mut child = Command::new("fzf")
            .args([
                "--height=40%",
                "--layout=reverse",
                "--border",
                "--prompt=dev > ",
                r#"--header=Enter: select / x: delete project"#,
                "--expect=enter,x",
                "--with-nth=1",
                "--delimiter=\t",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| UiError::ProcessError("Failed to get stdin".to_string()))?;

        for item in &items {
            stdin
                .write_all(format!("{}\n", item).as_bytes())
                .map_err(|e| UiError::ProcessError(e.to_string()))?;
        }
        let _ = stdin;

        let output = child
            .wait_with_output()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.is_empty() {
            return Ok(None);
        }

        let key = lines[0];
        let selection = lines.get(1).unwrap_or(&"");

        if selection.is_empty() {
            return Ok(None);
        }

        let selected_path = selection.split('\t').next().unwrap_or("");

        let index = projects
            .iter()
            .position(|p| p.display_name() == selected_path)
            .ok_or_else(|| UiError::ProcessError("Project not found".to_string()))?;

        let selection_type = if key == "x" {
            SelectionType::Delete
        } else {
            SelectionType::Enter
        };

        Ok(Some(ProjectSelection {
            index,
            selection_type,
        }))
    }

    fn tool_selection(&self, tools: &[Tool]) -> Result<Option<usize>> {
        check_fzf()?;

        let mut items: Vec<String> = tools
            .iter()
            .map(|t| {
                if t.recent > 0 {
                    format!("{} (use count: {})", t.name, t.recent)
                } else {
                    t.name.clone()
                }
            })
            .collect();

        items.push("[Add new tool...]".to_string());
        items.push("[Cancel]".to_string());

        let mut child = Command::new("fzf")
            .args([
                "--height=50%",
                "--layout=reverse",
                "--border",
                "--prompt=Select tool > ",
                "--with-nth=1",
                "--delimiter=\t",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| UiError::ProcessError("Failed to get stdin".to_string()))?;

        for item in &items {
            stdin
                .write_all(format!("{}\n", item).as_bytes())
                .map_err(|e| UiError::ProcessError(e.to_string()))?;
        }
        let _ = stdin;

        let output = child
            .wait_with_output()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let selected = stdout.trim();

        if selected.is_empty() || selected == "[Cancel]" {
            return Ok(None);
        }

        if selected == "[Add new tool...]" {
            return Ok(Some(usize::MAX));
        }

        let clean_name = selected
            .split('\t')
            .next()
            .unwrap_or(selected)
            .split('(')
            .next()
            .unwrap_or(selected)
            .trim()
            .to_string();

        Ok(tools.iter().position(|t| t.name == clean_name))
    }

    fn confirm_deletion(&self, project_name: &str) -> Result<bool> {
        check_fzf()?;

        let prompt = format!("Delete {} ? > ", project_name);

        let mut child = Command::new("fzf")
            .args([
                "--height=3",
                "--layout=reverse",
                "--border",
                &prompt,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| UiError::ProcessError("Failed to get stdin".to_string()))?;

        stdin
            .write_all(b"Yes\nNo\n")
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        let _ = stdin;

        let output = child
            .wait_with_output()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim() == "Yes")
    }

    fn add_tool_interactive(&self) -> Result<Option<(String, String)>> {
        print!("Tool name: ");
        std::io::stdout().flush().map_err(|e| UiError::ProcessError(e.to_string()))?;
        let mut name = String::new();
        std::io::stdin()
            .read_line(&mut name)
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        name = name.trim().to_string();

        if name.is_empty() {
            return Ok(None);
        }

        print!("Command template ($PROJECT_PATH and $PROJECT_NAME will be replaced): ");
        std::io::stdout()
            .flush()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        let mut command = String::new();
        std::io::stdin()
            .read_line(&mut command)
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        command = command.trim().to_string();

        if command.is_empty() {
            return Ok(None);
        }

        Ok(Some((name, command)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_type_equality() {
        assert_eq!(SelectionType::Enter, SelectionType::Enter);
        assert_eq!(SelectionType::Delete, SelectionType::Delete);
    }
}
