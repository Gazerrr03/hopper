use clap::{Parser, Subcommand, ValueEnum};
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

    /// Internal: write the selected project path for shell integration.
    #[arg(long, hide = true)]
    pub cwd_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 非交互式执行：hopper run <project> <tool>
    Run { project: String, tool: String },
    /// 交互式选择（默认行为）
    Interactive,
    /// Print shell integration for persistent cd support.
    Init {
        #[arg(value_enum)]
        shell: InitShell,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InitShell {
    Zsh,
    Bash,
    Fish,
    Powershell,
}

impl Cli {
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn is_interactive(&self) -> bool {
        match &self.command {
            Some(Commands::Interactive) => true,
            Some(Commands::Run { .. }) => false,
            Some(Commands::Init { .. }) => false,
            None => true,
        }
    }

    pub fn run_command(&self) -> Option<(&str, &str)> {
        match &self.command {
            Some(Commands::Run { project, tool }) => Some((project, tool)),
            _ => None,
        }
    }

    pub fn init_shell(&self) -> Option<InitShell> {
        match &self.command {
            Some(Commands::Init { shell }) => Some(*shell),
            _ => None,
        }
    }
}

pub fn shell_init_script(shell: InitShell) -> &'static str {
    match shell {
        InitShell::Zsh | InitShell::Bash => {
            r#"# Hopper shell integration: enables `hopper` to cd the current shell on [Cancel].
hopper() {
  local __hopper_cwd_file
  __hopper_cwd_file="$(mktemp "${TMPDIR:-/tmp}/hopper-cwd.XXXXXX")" || return

  command hopper --cwd-file "$__hopper_cwd_file" "$@"
  local __hopper_status=$?

  if [ $__hopper_status -eq 0 ] && [ -s "$__hopper_cwd_file" ]; then
    local __hopper_target
    __hopper_target="$(cat "$__hopper_cwd_file")"
    rm -f "$__hopper_cwd_file"
    cd -- "$__hopper_target"
    return $?
  fi

  rm -f "$__hopper_cwd_file"
  return $__hopper_status
}
"#
        }
        InitShell::Fish => {
            r#"# Hopper shell integration: enables `hopper` to cd the current shell on [Cancel].
function hopper
    set -l __hopper_tmpdir (set -q TMPDIR; and echo $TMPDIR; or echo /tmp)
    set -l __hopper_cwd_file (mktemp "$__hopper_tmpdir/hopper-cwd.XXXXXX")
    or return 1

    command hopper --cwd-file "$__hopper_cwd_file" $argv
    set -l __hopper_status $status

    if test $__hopper_status -eq 0; and test -s "$__hopper_cwd_file"
        set -l __hopper_target (cat "$__hopper_cwd_file")
        rm -f "$__hopper_cwd_file"
        cd "$__hopper_target"
        set __hopper_status $status
    end

    rm -f "$__hopper_cwd_file"
    return $__hopper_status
end
"#
        }
        InitShell::Powershell => {
            r#"# Hopper shell integration: enables `hopper` to cd the current shell on [Cancel].
function hopper {
    $hopperCwdFile = [System.IO.Path]::GetTempFileName()
    $hopperCommand = Get-Command hopper.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $hopperCommand) {
        $hopperCommand = Get-Command hopper -CommandType Application -ErrorAction Stop | Select-Object -First 1
    }

    & $hopperCommand.Source --cwd-file $hopperCwdFile @args
    $hopperStatus = $LASTEXITCODE

    if ($hopperStatus -eq 0 -and (Test-Path -LiteralPath $hopperCwdFile) -and (Get-Item -LiteralPath $hopperCwdFile).Length -gt 0) {
        $hopperTarget = Get-Content -LiteralPath $hopperCwdFile -Raw
        Remove-Item -LiteralPath $hopperCwdFile -Force -ErrorAction SilentlyContinue
        Set-Location -LiteralPath $hopperTarget.Trim()
        return
    }

    Remove-Item -LiteralPath $hopperCwdFile -Force -ErrorAction SilentlyContinue
    if ($null -ne $hopperStatus) {
        $global:LASTEXITCODE = $hopperStatus
    }
}
"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posix_init_script_contains_cwd_file_and_cd() {
        let script = shell_init_script(InitShell::Zsh);
        assert!(script.contains("--cwd-file"));
        assert!(script.contains("command hopper"));
        assert!(script.contains("cd --"));
    }

    #[test]
    fn test_bash_init_script_contains_cwd_file_and_cd() {
        let script = shell_init_script(InitShell::Bash);
        assert!(script.contains("--cwd-file"));
        assert!(script.contains("command hopper"));
        assert!(script.contains("cd --"));
    }

    #[test]
    fn test_fish_init_script_contains_cwd_file_and_cd() {
        let script = shell_init_script(InitShell::Fish);
        assert!(script.contains("--cwd-file"));
        assert!(script.contains("function hopper"));
        assert!(script.contains("cd \"$__hopper_target\""));
    }

    #[test]
    fn test_powershell_init_script_contains_cwd_file_and_cd() {
        let script = shell_init_script(InitShell::Powershell);
        assert!(script.contains("--cwd-file"));
        assert!(script.contains("function hopper"));
        assert!(script.contains("Set-Location -LiteralPath"));
    }
}
