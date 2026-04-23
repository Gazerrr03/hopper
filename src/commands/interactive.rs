use crate::core::cache::Cache;
use crate::core::config::Config;
use crate::core::project::{delete_project, discover_projects, sort_by_mru, Project};
use crate::error::Result;
use crate::ui::fzf::{OnboardingChoice, ProjectSelection, ProjectSelectionResult, SelectionType, UiBackend};
use std::fs;

pub struct InteractiveSession<'a> {
    pub config: &'a mut Config,
    pub cache: &'a mut Cache,
    pub ui: &'a dyn UiBackend,
    pub dry_run: bool,
}

impl<'a> InteractiveSession<'a> {
    pub fn run(&mut self) -> Result<bool> {
        if self.config.project_sets.is_empty() {
            self.run_onboarding()?;
        }

        let projects = discover_projects(&self.config.project_sets);
        let mut projects = projects;

        sort_by_mru(&mut projects, self.cache);

        if projects.is_empty() {
            println!("No projects found. Add project sets in config.");
            return Ok(false);
        }

        match self.ui.project_selection(&projects)? {
            Some(ProjectSelectionResult::Selected(selection)) => {
                self.handle_project_selection(&projects, selection)
            }
            Some(ProjectSelectionResult::NewProject(name)) => {
                self.handle_new_project(&name)
            }
            Some(ProjectSelectionResult::ManageProjectSets) => {
                if let Some(new_sets) =
                    self.ui.project_set_management(&self.config.project_sets)?
                {
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
                    Some(idx) => {
                        if idx == usize::MAX {
                            if let Some((name, command)) = self.ui.add_tool_interactive()? {
                                self.config.add_tool(name, command);
                                self.config.save()?;
                            }
                            return Ok(true);
                        }

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
        let base_set = self
            .config
            .project_sets
            .first()
            .ok_or_else(|| crate::error::ToolError::LaunchFailed("No project set configured".to_string()))?;

        let new_path = base_set.join(name);
        fs::create_dir_all(&new_path)?;

        self.cache.record_access(&new_path);
        self.cache.save()?;
        std::env::set_current_dir(&new_path)?;

        match self.ui.tool_selection(&self.config.tools)? {
            Some(idx) => {
                if idx == usize::MAX {
                    if let Some((tool_name, command)) = self.ui.add_tool_interactive()? {
                        self.config.add_tool(tool_name, command);
                        self.config.save()?;
                    }
                    return Ok(true);
                }

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
                // 直接进入主界面
            }
        }
        Ok(())
    }
}
