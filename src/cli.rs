use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hopper")]
#[command(version = "0.2.0")]
#[command(about = "A elegant project launcher", long_about = None)]
pub struct Cli {
    /// 环境变量覆盖配置文件路径
    #[arg(long, env = "HOPPER_CONFIG")]
    pub config: Option<PathBuf>,

    /// 环境变量覆盖缓存目录路径
    #[arg(long, env = "HOPPER_CACHE_DIR")]
    pub cache_dir: Option<PathBuf>,

    /// Dry-run 模式：仅打印命令不执行
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 非交互式执行：hopper run <project> <tool>
    Run {
        project: String,
        tool: String,
    },
    /// 交互式选择（默认行为）
    Interactive,
}

impl Cli {
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn is_interactive(&self) -> bool {
        match &self.command {
            Some(Commands::Interactive) => true,
            Some(Commands::Run { .. }) => false,
            None => true,
        }
    }

    pub fn run_command(&self) -> Option<(&str, &str)> {
        match &self.command {
            Some(Commands::Run { project, tool }) => Some((project, tool)),
            _ => None,
        }
    }
}
