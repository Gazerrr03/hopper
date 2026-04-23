# hopper

每次打开终端要 cd 好几层目录才能开始写代码？Hopper 帮你一步到位。

Hopper 是一个跨平台 CLI 项目启动器。输入 `hopper`，选项目、选工具，直接开干。

## 功能

- **项目集管理**：配置多个项目根目录（如 `~/Projects`、`~/Work`），自动扫描子文件夹
- **智能排序**：按修改时间 + 访问频率排序，最近用的项目永远在最上面
- **一键启动 coding agent**：选中项目后选择工具，直接在当前终端运行
- **新建项目**：在列表里直接输入一个不存在的名字，自动创建文件夹
- **管理项目集**：按 `m` 随时增删项目集路径
- **自定义工具**：支持添加任意 CLI 工具，支持 `$PROJECT_PATH` / `$PROJECT_NAME` 变量替换
- **快速删除**：按 `x` 删除项目，带二次确认防误触
- **非交互模式**：`hopper run hopper claude` 一步到位，跳过所有选择
- **跨平台**：macOS + Windows 通用

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

### 前置依赖

- [fzf](https://github.com/junegunn/fzf) — 交互选择器

```bash
# macOS
brew install fzf

# Linux
apt install fzf    # Debian/Ubuntu
pacman -S fzf     # Arch

# Windows
winget install fzf
```

## 使用

### 交互模式（默认）

```bash
hopper
```

#### 首次运行

第一次跑没有配置文件，会弹出 fzf 让你选择：

```
┌──────────────────────────┐
│ 首次运行：选择操作         │
│ > 绑定项目集...           │
│   跳过                    │
└──────────────────────────┘
```

选「绑定项目集...」，输入你的项目目录路径（比如 `~/Projects`），配置就保存好了。

选「跳过」的话会直接进入主界面，但因为没配项目集，列表为空会直接退出——所以**建议第一次先绑定**。

#### 选择项目

```
┌──────────────────────────────────────────┐
│ dev >                                    │
│ Enter: select / x: delete / m: manage    │
│ > hopper              5 min ago          │
│   my-app              2h ago             │
│   blog                3d ago             │
│   管理项目集...                           │
└──────────────────────────────────────────┘
```

- `↑` `↓` 移动光标，`Enter` 选中
- 直接打字可以模糊搜索
- 输入一个不存在的名字 = 在第一个项目集目录下新建文件夹

#### 选择工具

选中项目后进入工具选择：

```
┌──────────────────────────────────────┐
│ Select tool >                        │
│ > claude          (use count: 5)     │
│   codex           (use count: 2)     │
│   [Add new tool...]                  │
│   [Cancel]                           │
└──────────────────────────────────────┘
```

选中后工具直接接管当前终端会话。

### 非交互模式

跳过所有选择，直接指定项目和工具：

```bash
hopper run <项目名> <工具名>

# 例
hopper run hopper claude
hopper run my-app codex
```

项目名支持模糊匹配（精确匹配 > 前缀匹配 > 子串匹配）。

## 快捷键

| 按键 | 动作 |
|------|------|
| `↑` / `↓` | 上下移动 |
| `Enter` | 确认选择 |
| `x` | 删除项目（需二次确认） |
| `m` | 管理项目集（增删路径） |
| 直接输入 | 模糊搜索 / 新建项目 |
| `Esc` | 取消退出 |

## 命令行参数

| 参数 | 说明 |
|------|------|
| `hopper` | 交互模式（默认） |
| `hopper interactive` | 同上，显式指定 |
| `hopper run <project> <tool>` | 非交互模式，一步到位 |
| `--dry-run` | 只打印会执行的命令，不真正运行 |
| `--config <path>` | 指定配置文件路径（或设 `HOPPER_CONFIG` 环境变量） |
| `--cache-dir <path>` | 指定缓存目录路径（或设 `HOPPER_CACHE_DIR` 环境变量） |

## 配置

配置文件位置：
- **macOS / Linux**: `~/.config/hopper/config.json`
- **Windows**: `%APPDATA%\hopper\config.json`

### 手动编辑

```json
{
  "projectSets": [
    "/path/to/your/projects",
    "/path/to/work"
  ],
  "tools": [
    {"name": "claude", "command": "claude", "recent": 5},
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

## 构建

需要 [Rust](https://rustup.rs/) 环境：

```bash
cargo build --release
./target/release/hopper
```

## License

MIT
