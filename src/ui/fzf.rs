use crate::core::config::Tool;
use crate::core::project::Project;
use crate::error::{Result, UiError};
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Enter,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSelection {
    pub index: usize,
    pub selection_type: SelectionType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSelection {
    Tool(usize),
    AddNew,
    ProjectOnly,
}

pub trait UiBackend: Send + Sync {
    fn project_selection(&self, projects: &[Project]) -> Result<Option<ProjectSelectionResult>>;
    fn tool_selection(&self, tools: &[Tool]) -> Result<Option<ToolSelection>>;
    fn confirm_deletion(&self, name: &str) -> Result<bool>;
    fn add_tool_interactive(&self) -> Result<Option<(String, String)>>;
    fn onboarding_selection(&self) -> Result<Option<OnboardingChoice>>;
    fn project_set_management(&self, current_sets: &[PathBuf]) -> Result<Option<Vec<PathBuf>>>;
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

fn validate_terminal_support(
    stdin_is_terminal: bool,
    stdout_is_terminal: bool,
    term: &str,
) -> Result<()> {
    if !stdin_is_terminal || !stdout_is_terminal {
        return Err(UiError::UnsupportedTerminal(
            "Interactive mode requires a real terminal (TTY). Run `hopper` in your terminal, or use `hopper run <project> <tool>` for non-interactive launches.".to_string(),
        )
        .into());
    }

    if term.is_empty() || term == "dumb" {
        return Err(UiError::UnsupportedTerminal(
            "Interactive mode needs a terminal that supports fzf. The current TERM is not supported; open a normal shell and run `hopper` there.".to_string(),
        )
        .into());
    }

    Ok(())
}

fn ensure_fzf_terminal() -> Result<()> {
    let term = std::env::var("TERM").unwrap_or_default();
    validate_terminal_support(stdin().is_terminal(), stdout().is_terminal(), &term)
}

#[derive(Debug, Clone, Copy, Default)]
struct FzfOutputMode {
    print_query: bool,
    capture_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FzfResponse {
    key: Option<String>,
    query: Option<String>,
    selection: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FzfOutcome {
    Cancelled,
    Selection(FzfResponse),
}

const ADD_TOOL_LABEL: &str = "[Add new tool...]";
const CANCEL_TOOL_LABEL: &str = "[Cancel]";

fn build_process_error(code: Option<i32>, stderr: &str) -> UiError {
    let stderr = stderr.trim();
    let detail = if stderr.is_empty() {
        match code {
            Some(code) => format!("fzf exited with status {}", code),
            None => "fzf terminated unexpectedly".to_string(),
        }
    } else {
        stderr.to_string()
    };

    UiError::ProcessError(detail)
}

fn parse_fzf_output(
    code: Option<i32>,
    stdout: &str,
    stderr: &str,
    mode: FzfOutputMode,
) -> Result<FzfOutcome> {
    match code {
        Some(0) => {}
        Some(1) | Some(130) => return Ok(FzfOutcome::Cancelled),
        _ => return Err(build_process_error(code, stderr).into()),
    }

    let lines: Vec<&str> = stdout.lines().collect();
    if lines.is_empty() {
        return Err(UiError::ProcessError(
            "fzf exited successfully but did not return any selection data".to_string(),
        )
        .into());
    }

    let mut index = 0;
    let query = if mode.print_query {
        let value = lines
            .get(index)
            .map(|line| line.trim().to_string())
            .unwrap_or_default();
        index += 1;
        Some(value)
    } else {
        None
    };

    let key = if mode.capture_key {
        let value = lines
            .get(index)
            .map(|line| line.trim().to_string())
            .unwrap_or_default();
        index += 1;
        Some(value)
    } else {
        None
    };

    let selection = lines
        .get(index)
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty());

    if selection.is_none() && query.as_deref().unwrap_or_default().is_empty() {
        return Ok(FzfOutcome::Cancelled);
    }

    Ok(FzfOutcome::Selection(FzfResponse {
        key,
        query,
        selection,
    }))
}

fn run_fzf(items: &[String], args: &[&str], mode: FzfOutputMode) -> Result<FzfOutcome> {
    check_fzf()?;
    ensure_fzf_terminal()?;

    let mut child = Command::new("fzf")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| UiError::ProcessError(e.to_string()))?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| UiError::ProcessError("Failed to get stdin".to_string()))?;

    for item in items {
        stdin
            .write_all(format!("{}\n", item).as_bytes())
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
    }
    let _ = stdin;

    let output = child
        .wait_with_output()
        .map_err(|e| UiError::ProcessError(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_fzf_output(output.status.code(), &stdout, &stderr, mode)
}

fn project_index_by_display_name(projects: &[Project], selected_label: &str) -> Option<usize> {
    projects
        .iter()
        .position(|project| project.display_name() == selected_label)
}

fn parse_project_selection_response(
    projects: &[Project],
    response: FzfResponse,
) -> Option<ProjectSelectionResult> {
    let key = response.key.as_deref().unwrap_or_default();
    let query = response.query.as_deref().unwrap_or_default().trim();
    let selection = response.selection.as_deref().unwrap_or_default();
    let selected_label = selection.split('\t').next().unwrap_or(selection).trim();

    if key == "m" {
        return Some(ProjectSelectionResult::ManageProjectSets);
    }

    if let Some(index) = project_index_by_display_name(projects, selected_label) {
        let selection_type = if key == "x" {
            SelectionType::Delete
        } else {
            SelectionType::Enter
        };
        return Some(ProjectSelectionResult::Selected(ProjectSelection {
            index,
            selection_type,
        }));
    }

    if key == "enter" && !query.is_empty() && query != "管理项目集..." {
        return Some(ProjectSelectionResult::NewProject(query.to_string()));
    }

    if selected_label == "管理项目集..." {
        return Some(ProjectSelectionResult::ManageProjectSets);
    }

    None
}

fn parse_tool_selection_response(tools: &[Tool], response: FzfResponse) -> Option<ToolSelection> {
    let selected = response.selection.unwrap_or_default();

    if selected == CANCEL_TOOL_LABEL {
        return Some(ToolSelection::ProjectOnly);
    }

    if selected == ADD_TOOL_LABEL {
        return Some(ToolSelection::AddNew);
    }

    let clean_name = selected
        .split('\t')
        .next()
        .unwrap_or(&selected)
        .split('(')
        .next()
        .unwrap_or(&selected)
        .trim()
        .to_string();

    tools
        .iter()
        .position(|tool| tool.name == clean_name)
        .map(ToolSelection::Tool)
}

pub struct FzfBackend;

impl FzfBackend {
    pub fn new() -> Self {
        Self
    }

    fn add_project_set_interactive(
        &self,
        current_sets: &[PathBuf],
    ) -> Result<Option<Vec<PathBuf>>> {
        print!("输入项目集路径: ");
        stdout()
            .flush()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        let input = input.trim().to_string();

        if input.is_empty() {
            return Ok(None);
        }

        let path = shellexpand::full(&input)
            .map(|s| PathBuf::from(s.as_ref()))
            .unwrap_or_else(|_| PathBuf::from(&input));

        if !path.exists() || !path.is_dir() {
            println!("路径不存在或不是目录: {}", path.display());
            return Ok(None);
        }

        let mut new_sets = current_sets.to_vec();
        new_sets.push(path);
        Ok(Some(new_sets))
    }
}

impl Default for FzfBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl UiBackend for FzfBackend {
    fn project_selection(&self, projects: &[Project]) -> Result<Option<ProjectSelectionResult>> {
        let mut items: Vec<String> = vec!["管理项目集...".to_string()];
        items.extend(
            projects
                .iter()
                .map(|project| format!("{}\t{}", project.display_name(), project.mtime_str())),
        );

        if items.len() == 1 {
            return Ok(None);
        }

        let args = [
            "--height=40%",
            "--layout=reverse",
            "--border=rounded",
            "--prompt=dev > ",
            "--header=Enter: select / x: delete / m: manage / type: new project",
            "--expect=enter,x,m",
            "--print-query",
            "--bind=enter:accept-or-print-query",
            "--with-nth=1",
            "--delimiter=\t",
        ];

        match run_fzf(
            &items,
            &args,
            FzfOutputMode {
                print_query: true,
                capture_key: true,
            },
        )? {
            FzfOutcome::Cancelled => Ok(None),
            FzfOutcome::Selection(response) => {
                Ok(parse_project_selection_response(projects, response))
            }
        }
    }

    fn tool_selection(&self, tools: &[Tool]) -> Result<Option<ToolSelection>> {
        let mut items: Vec<String> = tools
            .iter()
            .map(|tool| {
                if tool.recent > 0 {
                    format!("{} (use count: {})", tool.name, tool.recent)
                } else {
                    tool.name.clone()
                }
            })
            .collect();

        items.push(ADD_TOOL_LABEL.to_string());
        items.push(CANCEL_TOOL_LABEL.to_string());

        let args = [
            "--height=50%",
            "--layout=reverse",
            "--border=rounded",
            "--prompt=Select tool > ",
            "--with-nth=1",
            "--delimiter=\t",
        ];

        match run_fzf(&items, &args, FzfOutputMode::default())? {
            FzfOutcome::Cancelled => Ok(None),
            FzfOutcome::Selection(response) => Ok(parse_tool_selection_response(tools, response)),
        }
    }

    fn confirm_deletion(&self, project_name: &str) -> Result<bool> {
        let prompt = format!("--prompt=Delete {} ? > ", project_name);
        let items = vec!["Yes".to_string(), "No".to_string()];
        let args = [
            "--height=3",
            "--layout=reverse",
            "--border=rounded",
            &prompt,
        ];

        match run_fzf(&items, &args, FzfOutputMode::default())? {
            FzfOutcome::Cancelled => Ok(false),
            FzfOutcome::Selection(response) => Ok(response.selection.as_deref() == Some("Yes")),
        }
    }

    fn add_tool_interactive(&self) -> Result<Option<(String, String)>> {
        print!("Tool name: ");
        stdout()
            .flush()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let mut name = String::new();
        stdin()
            .read_line(&mut name)
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        let name = name.trim().to_string();

        if name.is_empty() {
            return Ok(None);
        }

        print!("Command template ($PROJECT_PATH and $PROJECT_NAME will be replaced): ");
        stdout()
            .flush()
            .map_err(|e| UiError::ProcessError(e.to_string()))?;

        let mut command = String::new();
        stdin()
            .read_line(&mut command)
            .map_err(|e| UiError::ProcessError(e.to_string()))?;
        let command = command.trim().to_string();

        if command.is_empty() {
            return Ok(None);
        }

        Ok(Some((name, command)))
    }

    fn onboarding_selection(&self) -> Result<Option<OnboardingChoice>> {
        let items = vec!["绑定项目集...".to_string(), "跳过".to_string()];
        let args = [
            "--height=10",
            "--layout=reverse",
            "--border=rounded",
            "--prompt=选择 > ",
            "--header=首次运行：选择操作",
        ];

        match run_fzf(&items, &args, FzfOutputMode::default())? {
            FzfOutcome::Cancelled => Ok(None),
            FzfOutcome::Selection(response) => match response.selection.as_deref() {
                Some("绑定项目集...") => Ok(Some(OnboardingChoice::ConfigureProjectSets)),
                Some("跳过") => Ok(Some(OnboardingChoice::Skip)),
                _ => Ok(None),
            },
        }
    }

    fn project_set_management(&self, current_sets: &[PathBuf]) -> Result<Option<Vec<PathBuf>>> {
        let mut items: Vec<String> = current_sets
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        items.push("[添加新路径...]".to_string());

        let args = [
            "--height=50%",
            "--layout=reverse",
            "--border=rounded",
            "--prompt=项目集 > ",
            "--header=Enter: 确认 / x: 删除 / n: 新增",
            "--expect=enter,x,n",
            "--with-nth=1",
            "--delimiter=\t",
        ];

        match run_fzf(
            &items,
            &args,
            FzfOutputMode {
                print_query: false,
                capture_key: true,
            },
        )? {
            FzfOutcome::Cancelled => Ok(None),
            FzfOutcome::Selection(response) => {
                let key = response.key.as_deref().unwrap_or_default();
                let selected = response.selection.as_deref().unwrap_or_default();

                if key == "n" || selected == "[添加新路径...]" {
                    return self.add_project_set_interactive(current_sets);
                }

                if key == "x" {
                    if let Some(index) = items.iter().position(|item| item == selected) {
                        let mut new_sets = current_sets.to_vec();
                        if index < new_sets.len() {
                            new_sets.remove(index);
                        }
                        return Ok(Some(new_sets));
                    }
                }

                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_projects() -> Vec<Project> {
        vec![Project {
            path: PathBuf::from("/tmp/workspace/api"),
            mtime: None,
            base_path: PathBuf::from("/tmp/workspace"),
        }]
    }

    fn sample_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "claude".to_string(),
                command: "claude".to_string(),
                recent: 0,
            },
            Tool {
                name: "codex".to_string(),
                command: "codex".to_string(),
                recent: 3,
            },
        ]
    }

    #[test]
    fn test_selection_type_equality() {
        assert_eq!(SelectionType::Enter, SelectionType::Enter);
        assert_eq!(SelectionType::Delete, SelectionType::Delete);
    }

    #[test]
    fn test_parse_fzf_output_success_with_query_and_key() {
        let outcome = parse_fzf_output(
            Some(0),
            "newproj\nenter\n管理项目集...\n",
            "",
            FzfOutputMode {
                print_query: true,
                capture_key: true,
            },
        )
        .unwrap();

        assert_eq!(
            outcome,
            FzfOutcome::Selection(FzfResponse {
                key: Some("enter".to_string()),
                query: Some("newproj".to_string()),
                selection: Some("管理项目集...".to_string()),
            })
        );
    }

    #[test]
    fn test_parse_fzf_output_non_zero_is_error() {
        let err = parse_fzf_output(
            Some(2),
            "",
            "unknown option: --bad",
            FzfOutputMode::default(),
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "UI error: fzf process error: unknown option: --bad"
        );
    }

    #[test]
    fn test_parse_fzf_output_cancelled_when_aborted() {
        let outcome = parse_fzf_output(Some(130), "", "", FzfOutputMode::default()).unwrap();
        assert_eq!(outcome, FzfOutcome::Cancelled);
    }

    #[test]
    fn test_parse_project_selection_uses_existing_project_when_available() {
        let response = FzfResponse {
            key: Some("enter".to_string()),
            query: Some("api".to_string()),
            selection: Some("api\t1m ago".to_string()),
        };

        let parsed = parse_project_selection_response(&sample_projects(), response).unwrap();
        match parsed {
            ProjectSelectionResult::Selected(selection) => {
                assert_eq!(selection.index, 0);
                assert_eq!(selection.selection_type, SelectionType::Enter);
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn test_parse_project_selection_creates_new_project_from_query() {
        let response = FzfResponse {
            key: Some("enter".to_string()),
            query: Some("newproj".to_string()),
            selection: Some("管理项目集...".to_string()),
        };

        let parsed = parse_project_selection_response(&sample_projects(), response);
        assert_eq!(
            parsed,
            Some(ProjectSelectionResult::NewProject("newproj".to_string()))
        );
    }

    #[test]
    fn test_project_selection_manage_shortcut_wins() {
        let response = FzfResponse {
            key: Some("m".to_string()),
            query: Some(String::new()),
            selection: Some("api\t1m ago".to_string()),
        };

        let parsed = parse_project_selection_response(&sample_projects(), response);
        assert_eq!(parsed, Some(ProjectSelectionResult::ManageProjectSets));
    }

    #[test]
    fn test_parse_tool_selection_cancel_opens_project_only() {
        let response = FzfResponse {
            key: None,
            query: None,
            selection: Some(CANCEL_TOOL_LABEL.to_string()),
        };

        let parsed = parse_tool_selection_response(&sample_tools(), response);
        assert_eq!(parsed, Some(ToolSelection::ProjectOnly));
    }

    #[test]
    fn test_parse_tool_selection_add_new_tool() {
        let response = FzfResponse {
            key: None,
            query: None,
            selection: Some(ADD_TOOL_LABEL.to_string()),
        };

        let parsed = parse_tool_selection_response(&sample_tools(), response);
        assert_eq!(parsed, Some(ToolSelection::AddNew));
    }

    #[test]
    fn test_parse_tool_selection_strips_use_count() {
        let response = FzfResponse {
            key: None,
            query: None,
            selection: Some("codex (use count: 3)".to_string()),
        };

        let parsed = parse_tool_selection_response(&sample_tools(), response);
        assert_eq!(parsed, Some(ToolSelection::Tool(1)));
    }

    #[test]
    fn test_validate_terminal_support_rejects_non_tty() {
        let err = validate_terminal_support(false, true, "xterm-256color").unwrap_err();
        assert_eq!(
            err.to_string(),
            "UI error: Interactive mode requires a real terminal (TTY). Run `hopper` in your terminal, or use `hopper run <project> <tool>` for non-interactive launches."
        );
    }

    #[test]
    fn test_validate_terminal_support_rejects_dumb_term() {
        let err = validate_terminal_support(true, true, "dumb").unwrap_err();
        assert_eq!(
            err.to_string(),
            "UI error: Interactive mode needs a terminal that supports fzf. The current TERM is not supported; open a normal shell and run `hopper` there."
        );
    }
}
