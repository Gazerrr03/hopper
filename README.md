# hopper

跨平台 CLI 项目启动器。在任意终端里快速切换项目目录并启动你喜欢的 coding agent（Claude Code / Codex 等）。

## 功能

- **多项目集**：支持配置多个项目根目录（如 `~/Projects`、`~/Work`）
- **按修改时间排序**：最近活跃的项目排在最上面
- **一键启动 coding agent**：选中项目后选择工具，直接在当前终端运行
- **自定义工具**：支持添加任意 CLI 工具，支持 `$PROJECT_PATH` / `$PROJECT_NAME` 变量替换
- **跨平台**：macOS + Windows 通用
- **快速删除**：按 `x` 删除项目，带二次确认防误触

## 安装

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/qizhidong/hopper/main/install.sh | sh
```

### Windows

PowerShell 中运行：

```powershell
irm https://raw.githubusercontent.com/qizhidong/hopper/main/install.ps1 | iex
```

## 使用

```bash
hopper
```

### 首次运行

首次运行会进入引导配置，输入项目集路径即可：

```
First run setup...

Enter project set paths (e.g. ~/Projects, ~/Work):
Press Enter to confirm, empty to skip

Project set path: ~/Projects
Added: /Users/you/Projects
Add more project sets? (y/n): n
Config saved!
```

### 操作流程

```
1. 选择项目（上下键移动，模糊搜索）
   - Enter 选中，进入工具选择
   - x 删除项目（带确认）

2. 选择工具
   - Enter 启动工具（接管当前终端会话）
   - [Add new tool...] 添加自定义工具
   - [Cancel] 返回
```

### 快捷键

| 按键 | 动作 |
|------|------|
| `↑` / `↓` | 上/下移动 |
| `Enter` | 确认选择 |
| `x` | 删除项目（需二次确认） |
| `Esc` | 取消退出 |

## 配置

配置文件位于：
- **macOS / Linux**: `~/.config/hopper/config.json`
- **Windows**: `%APPDATA%\hopper\config.json`

### 手动编辑配置

```json
{
  "projectSets": [
    "/path/to/your/projects",
    "/path/to/work"
  ],
  "tools": [
    {"name": "claude code", "command": "claude", "recent": 5},
    {"name": "codex", "command": "codex", "recent": 2}
  ]
}
```

### 工具变量

工具命令支持以下变量自动替换：

| 变量 | 含义 |
|------|------|
| `$PROJECT_PATH` | 项目完整路径 |
| `$PROJECT_NAME` | 项目文件夹名 |

例：`claude --project-path $PROJECT_PATH`

## 依赖

- [fzf](https://github.com/junegunn/fzf) — 项目选择器（需自行安装）

  ```bash
  # macOS
  brew install fzf

  # Linux
  apt install fzf    # Debian/Ubuntu
  pacman -S fzf     # Arch

  # Windows
  winget install fzf
  ```

## 构建

```bash
cargo build --release
./target/release/hopper
```

## License

MIT
