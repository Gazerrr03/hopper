mod config;
mod project;
mod tool;
mod ui;

use crate::config::Config;
use crate::project::{delete_project, discover_projects};
use std::io::Write;

fn main() {
    let mut config = Config::load();

    // Check if project sets are configured
    if config.project_sets.is_empty() {
        run_onboarding(&mut config);
    }

    loop {
        let projects = discover_projects(&config.project_sets);

        match ui::project_selection(&projects) {
            Some((index, deleted)) => {
                let proj = &projects[index];

                if deleted {
                    // x was pressed - confirm deletion
                    let name = proj.path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if ui::confirm_deletion(&name) {
                        match delete_project(proj) {
                            Ok(_) => println!("Deleted: {}", proj.display_name()),
                            Err(e) => eprintln!("Delete failed: {}", e),
                        }
                    }
                    // Reload projects list (deleted item will be gone)
                    continue;
                }

                // Project selected - change to that directory
                std::env::set_current_dir(&proj.path).expect("Failed to change directory");

                // Tool selection
                match ui::tool_selection(&config.tools) {
                    Some(idx) => {
                        if idx == usize::MAX {
                            // Add new tool
                            ui::add_tool_interactive(&mut config);
                            config.save();
                            continue;
                        }

                        let tool_name = config.tools[idx].name.clone();

                        // Update recent count
                        if let Some(t) = config.tools.iter_mut().find(|t| t.name == tool_name) {
                            t.recent += 1;
                        }
                        config.save();

                        // Launch tool
                        tool::launch_tool(&config.tools[idx], &proj.path);

                        // Tool launched - exit the loop (user will return to their shell)
                        break;
                    }
                    None => {
                        // Cancel - stay in loop
                        continue;
                    }
                }
            }
            None => {
                // Empty list or ESC - exit
                break;
            }
        }
    }
}

fn run_onboarding(config: &mut Config) {
    println!("First run setup...");
    println!();
    println!("Enter project set paths (e.g. ~/Projects, ~/Work):");
    println!("Press Enter to confirm, empty to skip");
    println!();

    loop {
        print!("Project set path: ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();

        if input.is_empty() {
            break;
        }

        let path = shellexpand::full(&input)
            .map(|s| s.into_owned())
            .unwrap_or(input);

        let path = std::path::PathBuf::from(&path);

        if path.exists() && path.is_dir() {
            config.project_sets.push(path.clone());
            println!("Added: {}", path.display());
        } else {
            println!("Path does not exist or is not a directory: {}", path.display());
        }

        print!("Add more project sets? (y/n): ");
        std::io::stdout().flush().unwrap();
        let mut more = String::new();
        std::io::stdin().read_line(&mut more).unwrap();
        if !more.trim().to_lowercase().starts_with('y') {
            break;
        }
    }

    if !config.project_sets.is_empty() {
        config.save();
        println!("Config saved!");
    }
}
