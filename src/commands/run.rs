use crate::core::config::Config;
use crate::core::project::{discover_projects, find_project};
use crate::core::tool::launch_tool;
use crate::error::{ProjectError, Result, ToolError};

pub struct RunCommand<'a> {
    pub config: &'a Config,
    pub dry_run: bool,
}

impl<'a> RunCommand<'a> {
    pub fn execute(&self, project_name: &str, tool_name: &str) -> Result<()> {
        let projects = discover_projects(&self.config.project_sets);

        let project = find_project(project_name, &projects)
            .ok_or_else(|| ProjectError::NotFound(project_name.to_string()))?;

        let tool = self
            .config
            .find_tool(tool_name)
            .ok_or_else(|| ToolError::NotFound(tool_name.to_string()))?;

        if !self.dry_run {
            println!("Launching {} in {}...", tool.name, project.display_name());
        }

        launch_tool(tool, &project.path, self.dry_run)?;

        Ok(())
    }
}
