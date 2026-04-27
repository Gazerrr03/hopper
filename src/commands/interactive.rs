use crate::core::cache::Cache;
use crate::core::config::Config;
use crate::core::project::{delete_project, discover_projects, sort_by_mru, Project};
use crate::error::Result;
use crate::ui::fzf::{
    OnboardingChoice, ProjectSelection, ProjectSelectionResult, SelectionType, ToolSelection,
    UiBackend,
};
use std::fs;
use std::path::{Path, PathBuf};

pub struct InteractiveSession<'a> {
    pub config: &'a mut Config,
    pub cache: &'a mut Cache,
    pub ui: &'a dyn UiBackend,
    pub dry_run: bool,
    pub is_first_run: bool,
    pub cwd_file: Option<PathBuf>,
}

impl<'a> InteractiveSession<'a> {
    pub fn run(&mut self) -> Result<bool> {
        if self.config.project_sets.is_empty() {
            if self.is_first_run {
                println!("No hopper config found. Starting first-run setup...");
            }
            self.run_onboarding()?;
        }

        let projects = discover_projects(&self.config.project_sets);
        let mut projects = projects;

        sort_by_mru(&mut projects, self.cache);

        if projects.is_empty() {
            if self.config.project_sets.is_empty() {
                println!(
                    "No project sets configured. Run `hopper` in a regular terminal to bind one, or edit ~/.config/hopper/config.json."
                );
            } else {
                println!(
                    "No projects found under the configured project sets. Add a project directory or update your hopper config."
                );
            }
            return Ok(false);
        }

        match self.ui.project_selection(&projects)? {
            Some(ProjectSelectionResult::Selected(selection)) => {
                self.handle_project_selection(&projects, selection)
            }
            Some(ProjectSelectionResult::NewProject(name)) => self.handle_new_project(&name),
            Some(ProjectSelectionResult::ManageProjectSets) => {
                if let Some(new_sets) = self.ui.project_set_management(&self.config.project_sets)? {
                    self.config.project_sets = new_sets;
                    self.config.save()?;
                }
                self.run()
            }
            None => Ok(false),
        }
    }

    fn handle_project_selection(
        &mut self,
        projects: &[Project],
        selection: ProjectSelection,
    ) -> Result<bool> {
        let proj = &projects[selection.index];

        match selection.selection_type {
            SelectionType::Delete => {
                let name = proj
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if self.ui.confirm_deletion(&name)? {
                    delete_project(proj)?;
                    println!("Deleted: {}", proj.display_name());
                }
                Ok(true)
            }
            SelectionType::Enter => {
                self.cache.record_access(&proj.path);
                self.cache.save()?;

                std::env::set_current_dir(&proj.path)?;

                match self.ui.tool_selection(&self.config.tools)? {
                    Some(selection) => {
                        if selection == ToolSelection::AddNew {
                            if let Some((name, command)) = self.ui.add_tool_interactive()? {
                                self.config.add_tool(name, command);
                                self.config.save()?;
                            }
                            return Ok(true);
                        }

                        if selection == ToolSelection::ProjectOnly {
                            self.open_project_only(&proj.path)?;
                            return Ok(false);
                        }

                        let ToolSelection::Tool(idx) = selection else {
                            return Ok(true);
                        };

                        let tool_name = self.config.tools[idx].name.clone();
                        self.config.increment_tool_usage(&tool_name);
                        self.config.save()?;

                        let tool = &self.config.tools[idx];
                        crate::core::tool::launch_tool(tool, &proj.path, self.dry_run)?;

                        Ok(false)
                    }
                    None => Ok(true),
                }
            }
        }
    }

    fn handle_new_project(&mut self, name: &str) -> Result<bool> {
        let base_set = self.config.project_sets.first().ok_or_else(|| {
            crate::error::ToolError::LaunchFailed("No project set configured".to_string())
        })?;

        let new_path = base_set.join(name);
        fs::create_dir_all(&new_path)?;

        self.cache.record_access(&new_path);
        self.cache.save()?;
        std::env::set_current_dir(&new_path)?;

        match self.ui.tool_selection(&self.config.tools)? {
            Some(selection) => {
                if selection == ToolSelection::AddNew {
                    if let Some((tool_name, command)) = self.ui.add_tool_interactive()? {
                        self.config.add_tool(tool_name, command);
                        self.config.save()?;
                    }
                    return Ok(true);
                }

                if selection == ToolSelection::ProjectOnly {
                    self.open_project_only(&new_path)?;
                    return Ok(false);
                };

                let ToolSelection::Tool(idx) = selection else {
                    return Ok(true);
                };

                let tool_name = self.config.tools[idx].name.clone();
                self.config.increment_tool_usage(&tool_name);
                self.config.save()?;

                let tool = &self.config.tools[idx];
                crate::core::tool::launch_tool(tool, &new_path, self.dry_run)?;
                Ok(false)
            }
            None => Ok(true),
        }
    }

    fn open_project_only(&self, project_path: &Path) -> Result<()> {
        if let Some(cwd_file) = &self.cwd_file {
            let target = project_path
                .canonicalize()
                .unwrap_or_else(|_| project_path.to_path_buf());

            if self.dry_run {
                println!("[Dry-run] Would write cwd target: {}", target.display());
                return Ok(());
            }

            fs::write(cwd_file, target.to_string_lossy().as_bytes())?;
            return Ok(());
        }

        crate::core::tool::open_shell(project_path, self.dry_run)
    }

    fn run_onboarding(&mut self) -> Result<()> {
        match self.ui.onboarding_selection()? {
            Some(OnboardingChoice::ConfigureProjectSets) => {
                if let Some(new_sets) = self.ui.project_set_management(&[])? {
                    self.config.project_sets = new_sets;
                    self.config.save()?;
                    println!("配置已保存！");
                }
            }
            Some(OnboardingChoice::Skip) | None => {
                println!(
                    "Skipped project-set setup. Run `hopper` again in a regular terminal when you want to bind project folders."
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Tool;
    use std::env;
    use std::sync::Mutex;
    use tempfile::TempDir;

    struct StubUi {
        project_selection: Mutex<Option<ProjectSelectionResult>>,
        tool_selection: Mutex<Option<ToolSelection>>,
    }

    impl StubUi {
        fn new(
            project_selection: Option<ProjectSelectionResult>,
            tool_selection: Option<ToolSelection>,
        ) -> Self {
            Self {
                project_selection: Mutex::new(project_selection),
                tool_selection: Mutex::new(tool_selection),
            }
        }
    }

    impl UiBackend for StubUi {
        fn project_selection(
            &self,
            _projects: &[Project],
        ) -> Result<Option<ProjectSelectionResult>> {
            Ok(self.project_selection.lock().unwrap().take())
        }

        fn tool_selection(&self, _tools: &[Tool]) -> Result<Option<ToolSelection>> {
            Ok(self.tool_selection.lock().unwrap().take())
        }

        fn confirm_deletion(&self, _name: &str) -> Result<bool> {
            Ok(false)
        }

        fn add_tool_interactive(&self) -> Result<Option<(String, String)>> {
            Ok(None)
        }

        fn onboarding_selection(&self) -> Result<Option<OnboardingChoice>> {
            Ok(None)
        }

        fn project_set_management(
            &self,
            _current_sets: &[PathBuf],
        ) -> Result<Option<Vec<PathBuf>>> {
            Ok(None)
        }
    }

    fn config_with_project_set(project_set: PathBuf) -> Config {
        Config {
            project_sets: vec![project_set],
            tools: Vec::new(),
        }
    }

    fn temp_cache(temp_dir: &TempDir) -> Cache {
        Cache::new(Some(temp_dir.path().join("cache"))).unwrap()
    }

    #[test]
    fn test_project_only_writes_selected_project_to_cwd_file() {
        let original_cwd = env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_set = temp_dir.path().join("projects");
        let project = project_set.join("api");
        fs::create_dir_all(&project).unwrap();

        let cwd_file = temp_dir.path().join("cwd-target");
        let mut config = config_with_project_set(project_set);
        let mut cache = temp_cache(&temp_dir);
        let ui = StubUi::new(
            Some(ProjectSelectionResult::Selected(ProjectSelection {
                index: 0,
                selection_type: SelectionType::Enter,
            })),
            Some(ToolSelection::ProjectOnly),
        );

        let mut session = InteractiveSession {
            config: &mut config,
            cache: &mut cache,
            ui: &ui,
            dry_run: false,
            is_first_run: false,
            cwd_file: Some(cwd_file.clone()),
        };

        let result = session.run().unwrap();
        env::set_current_dir(original_cwd).unwrap();

        assert!(!result);
        assert_eq!(
            fs::read_to_string(cwd_file).unwrap(),
            project.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn test_project_only_writes_new_project_to_cwd_file() {
        let original_cwd = env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_set = temp_dir.path().join("projects");
        fs::create_dir_all(&project_set).unwrap();
        let new_project = project_set.join("new-app");

        let cwd_file = temp_dir.path().join("cwd-target");
        let mut config = config_with_project_set(project_set);
        let mut cache = temp_cache(&temp_dir);
        let ui = StubUi::new(None, Some(ToolSelection::ProjectOnly));

        let mut session = InteractiveSession {
            config: &mut config,
            cache: &mut cache,
            ui: &ui,
            dry_run: false,
            is_first_run: false,
            cwd_file: Some(cwd_file.clone()),
        };

        let result = session.handle_new_project("new-app").unwrap();
        env::set_current_dir(original_cwd).unwrap();

        assert!(!result);
        assert_eq!(
            fs::read_to_string(cwd_file).unwrap(),
            new_project.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn test_aborted_tool_selection_does_not_write_cwd_file() {
        let original_cwd = env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_set = temp_dir.path().join("projects");
        let project = project_set.join("api");
        fs::create_dir_all(&project).unwrap();

        let cwd_file = temp_dir.path().join("cwd-target");
        fs::write(&cwd_file, "").unwrap();
        let mut config = config_with_project_set(project_set);
        let mut cache = temp_cache(&temp_dir);
        let ui = StubUi::new(
            Some(ProjectSelectionResult::Selected(ProjectSelection {
                index: 0,
                selection_type: SelectionType::Enter,
            })),
            None,
        );

        let mut session = InteractiveSession {
            config: &mut config,
            cache: &mut cache,
            ui: &ui,
            dry_run: false,
            is_first_run: false,
            cwd_file: Some(cwd_file.clone()),
        };

        let result = session.run().unwrap();
        env::set_current_dir(original_cwd).unwrap();

        assert!(result);
        assert_eq!(fs::read_to_string(cwd_file).unwrap(), "");
    }
}
