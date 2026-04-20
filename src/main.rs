mod cli;
mod commands;
mod core;
mod error;
mod ui;

use clap::Parser;
use crate::cli::Cli;
use crate::commands::interactive::InteractiveSession;
use crate::commands::run::RunCommand;
use crate::core::cache::Cache;
use crate::core::config::Config;
use crate::error::Result;
use crate::ui::fzf::FzfBackend;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut config = Config::load_with_override(cli.config.clone())?;
    let mut cache = Cache::new(cli.cache_dir.clone())?;

    let ui: FzfBackend = FzfBackend::new();

    if cli.is_interactive() {
        let mut session = InteractiveSession {
            config: &mut config,
            cache: &mut cache,
            ui: &ui,
            dry_run: cli.is_dry_run(),
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
