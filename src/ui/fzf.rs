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

#[derive(Debug, Clone)]
pub enum ProjectSelectionResult {
    Selected(ProjectSelection),
    NewProject(String),
    ManageProjectSets,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnboardingChoice {
    ConfigureProjectSets,
    Skip,
}

pub trait UiBackend: Send + Sync {
    fn project_selection(&self, projects: &[Project]) -> Result<Option<ProjectSelectionResult>>;
    fn tool_selection(&self, tools: &[Tool]) -> Result<Option<usize>>;
    fn confirm_deletion(&self, name: &str) -> Result<bool>;
    fn add_tool_interactive(&self) -> Result<Option<(String, String)>>;
    fn onboarding_selection(&self) -> Result<Option<OnboardingChoice>>;
    fn project_set_management(&self, current_sets: &[std::path::PathBuf]) -> Result<Option<Vec<std::path::PathBuf>>>;
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
    fn project_selection(&self, projects: &[Project]) -> Result<Option<ProjectSelectionResult>> {
        check_fzf()?;

        let mut items: Vec<String> = vec!["管理项目集...".to_string()];
        items.extend(
            projects
                .iter()
                .map(|p| format!("{}\t{}", p.display_name(), p.mtime_str())),
        );

        if items.len() == 1 {
            return Ok(None);
        }

        let mut child = Command::new("fzf")
            .args([
                "--height=40%",
                "--layout=reverse",
                "--border",
                "--prompt=dev > ",
                r#"--header=Enter: select / x: delete / m: manage / type: new project"#,
                "--expect=enter,x,m",
                "--custom",
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

        // Check if user selected "管理项目集..."
        if *selection == "管理项目集..." || key == "m" {
            return Ok(Some(ProjectSelectionResult::ManageProjectSets));
        }

        let selected_path = selection.split('\t').next().unwrap_or("");

        // Check if selection matches an existing project
        if let Some(index) = projects.iter().position(|p| p.display_name() == selected_path) {
            let selection_type = if key == "x" {
                SelectionType::Delete
            } else {
                SelectionType::Enter
            };
            Ok(Some(ProjectSelectionResult::Selected(ProjectSelection {
                index,
                selection_type,
            })))
        } else if selected_path.is_empty() {
            Ok(None)
        } else {
            // User typed a new project name
            Ok(Some(ProjectSelectionResult::NewProject(selected_path.to_string())))
        }
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

    fn onboarding_selection(&self) -> Result<Option<OnboardingChoice>> {
        check_fzf()?;

        let items = vec!["绑定项目集...", "跳过"];

        let mut child = Command::new("fzf")
            .args([
                "--height=10",
                "--layout=reverse",
                "--border",
                "--prompt=选择 > ",
                "--header=首次运行：选择操作",
                "--expect=enter",
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

        let selected = lines.get(1).unwrap_or(&"");

        if selected.is_empty() {
            return Ok(None);
        }

        match *selected {
            "绑定项目集..." => Ok(Some(OnboardingChoice::ConfigureProjectSets)),
            "跳过" => Ok(Some(OnboardingChoice::Skip)),
            _ => Ok(None),
        }
    }

    fn project_set_management(&self, current_sets: &[std::path::PathBuf]) -> Result<Option<Vec<std::path::PathBuf>>> {
        check_fzf()?;

        let mut items: Vec<String> = current_sets
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        items.push("[添加新路径...]".to_string());

        let mut child = Command::new("fzf")
            .args([
                "--height=50%",
                "--layout=reverse",
                "--border",
                "--prompt=项目集 > ",
                "--header=Enter: 确认 / x: 删除 / n: 新增",
                "--expect=enter,x,n",
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
        let selected = lines.get(1).unwrap_or(&"");

        if selected.is_empty() {
            return Ok(None);
        }

        if *selected == "[添加新路径...]" {
            return self.add_project_set_interactive(current_sets);
        }

        let index = items.iter().position(|i| i == selected);

        if key == "x" {
            if let Some(idx) = index {
                let mut new_sets = current_sets.to_vec();
                if idx < new_sets.len() {
                    new_sets.remove(idx);
                }
                return Ok(Some(new_sets));
            }
        }

        if key == "n" {
            return self.add_project_set_interactive(current_sets);
        }

        Ok(None)
    }
}

impl FzfBackend {
    fn add_project_set_interactive(&self, current_sets: &[std::path::PathBuf]) -> Result<Option<Vec<std::path::PathBuf>>> {
        print!("输入项目集路径: ");
        std::io::stdout().flush().map_err(|e| UiError::ProcessError(e.to_string()))?;
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        let input = input.trim().to_string();

        if input.is_empty() {
            return Ok(None);
        }

        let path = shellexpand::full(&input)
            .map(|s| std::path::PathBuf::from(s.as_ref()))
            .unwrap_or_else(|_| std::path::PathBuf::from(&input));

        if !path.exists() || !path.is_dir() {
            println!("路径不存在或不是目录: {}", path.display());
            return Ok(None);
        }

        let mut new_sets = current_sets.to_vec();
        new_sets.push(path);
        Ok(Some(new_sets))
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
