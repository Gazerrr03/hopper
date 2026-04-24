mod cli;
mod commands;
mod core;
mod error;
mod ui;

use crate::cli::Cli;
use crate::commands::interactive::InteractiveSession;
use crate::commands::run::RunCommand;
use crate::core::cache::Cache;
use crate::core::config::Config;
use crate::error::Result;
use crate::ui::fzf::FzfBackend;
use clap::Parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_path = Config::resolve_path(cli.config.clone());
    let is_first_run = !config_path.exists();

    let mut config = Config::load_from_path(&config_path)?;
    let mut cache = Cache::new(cli.cache_dir.clone())?;

    let ui: FzfBackend = FzfBackend::new();

    if cli.is_interactive() {
        let mut session = InteractiveSession {
            config: &mut config,
            cache: &mut cache,
            ui: &ui,
            dry_run: cli.is_dry_run(),
            is_first_run,
        };
        session.run()?;
    } else if let Some((project, tool)) = cli.run_command() {
        let cmd = RunCommand {
            config: &config,
            dry_run: cli.is_dry_run(),
        };
        cmd.execute(project, tool)?;
    }

    Ok(())
}
