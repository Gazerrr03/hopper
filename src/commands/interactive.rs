use crate::core::cache::Cache;
use crate::core::config::Config;
use crate::core::project::{delete_project, discover_projects, sort_by_mru, Project};
use crate::error::Result;
use crate::ui::fzf::{ProjectSelection, SelectionType, UiBackend};
use std::io::Write;
use std::path::PathBuf;

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
            Some(selection) => self.handle_project_selection(&projects, selection),
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

    fn run_onboarding(&mut self) -> Result<()> {
        println!("First run setup...");
        println!();
        println!("Enter project set paths (e.g. ~/Projects, ~/Work):");
        println!("Press Enter to confirm, empty to skip");
        println!();

        loop {
            print!("Project set path: ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim().to_string();

            if input.is_empty() {
                break;
            }

            let path = shellexpand::full(&input)
                .map(|s| PathBuf::from(s.as_ref()))
                .unwrap_or_else(|_| PathBuf::from(&input));

            if path.exists() && path.is_dir() {
                self.config.project_sets.push(path.clone());
                println!("Added: {}", path.display());
            } else {
                println!(
                    "Path does not exist or is not a directory: {}",
                    path.display()
                );
            }

            print!("Add more project sets? (y/n): ");
            std::io::stdout().flush()?;
            let mut more = String::new();
            std::io::stdin().read_line(&mut more)?;
            if !more.trim().to_lowercase().starts_with('y') {
                break;
            }
        }

        if !self.config.project_sets.is_empty() {
            self.config.save()?;
            println!("Config saved!");
        }

        Ok(())
    }
}
