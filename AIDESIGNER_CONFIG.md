# AiDesigner — Agent 配置参考

> **用途**: 告诉 coding agent 如何安装、配置 AiDesigner，并明确它的能力边界。
> **适用对象**: Claude Code / Codex CLI / OpenCode / 任意 AI coding agent。

---

## 1. 一句话概述

AiDesigner 是一个**项目级 Prompt Agent 框架**，通过预设角色 Prompt + MCP 工具，让 AI 以"设计师/PM/架构师"等身份与用户对话，引导完成从想法到 UI 设计再到开发规划的全流程。

---

## 2. 安装与配置

### 前置条件

- Node.js ≥ 20.10.0
- 已安装以下任一 CLI：`claude` (Claude Code)、`codex`、`opencode`

### 一行初始化

```bash
npx aidesigner@latest start
```

这个命令自动完成：创建目录结构 → 安装依赖 → 配置 `.mcp.json` → 启动对话 CLI。

### 分步操作（如需手动控制）

```bash
npx aidesigner@latest init      # 1. 初始化项目结构
npm install                      # 2. 安装依赖
npm run aidesigner:claude        # 3. 启动 Claude CLI
# 或
npm run aidesigner:codex         #   启动 Codex CLI
```

### 选择 LLM 提供商

```bash
npx aidesigner@latest start --assistant=claude           # 默认 Anthropic
npx aidesigner@latest start --assistant=claude --glm     # 使用 GLM (ZhipuAI)
npx aidesigner@latest start --assistant=codex            # Codex + OpenAI
npx aidesigner@latest start --assistant=opencode --glm   # OpenCode + GLM
```

### 初始化后项目结构

```
your-project/
├── .mcp.json              ← MCP 配置，注册 aidesigner 工具
├── .aidesigner/           ← 项目元数据 & 对话状态
├── docs/                  ← AI Agent 产出文档的落点
│   ├── prd/
│   ├── architecture/
│   ├── stories/
│   └── qa/
└── package.json           ← 新增 aidesigner 依赖和 npm scripts
```

### 可选 MCP 扩展（安装时可勾选）

| 扩展 | 作用 |
|------|------|
| `chrome-devtools` | 从参考 URL 自动提取配色、字体、CSS token |
| `shadcn-ui` | 生成 shadcn/ui 组件代码 |
| `github` | GitHub API 操作（需 Personal Access Token） |
| `tweakcn` | Tailwind 主题同步 |

### 环境变量（仅 GLM 模式需要）

```bash
AGILAI_GLM_API_KEY=your-zhipu-api-key
AGILAI_GLM_BASE_URL=https://open.bigmodel.cn/api/paas/v4
```

Anthropic 模式无需额外配置——直接使用 Claude CLI 已有认证。

---

## 3. 工具 & 能力清单

### 可用角色（`@角色名` 激活）

| 角色 | 职责 | 产出 |
|------|------|------|
| `@analyst` | 需求分析、头脑风暴 | `docs/brief.md` |
| `@pm` | 产品需求文档 | `docs/prd.md` |
| `@ux-expert` | UI/UX 规格、前端规范 | `docs/front-end-spec.md` |
| `@architect` | 技术架构设计 | `docs/architecture.md` |
| `@po` | 文档验证 & 拆分 | sharded docs |
| `@sm` | 用户故事拆分 | `docs/stories/*.md` |
| `@dev` | 代码实现 | 具体代码文件 |
| `@qa` | 代码审查 & 质量门禁 | QA 报告 |
| `@ui-designer-liaison` | **交互式 UI 设计引导（核心）** | `docs/ui/ui-designer-screen-prompts.md` |
| `@quick-designer` | 快速 UI 生成（直接出 HTML/CSS） | `*.html` + 浏览器预览 |

### 角色内命令（`*` 触发）

| 命令 | 适用角色 | 作用 |
|------|----------|------|
| `*discover-journey` | ui-designer-liaison | 6 阶段交互式用户旅程映射 |
| `*assemble-prompts` | ui-designer-liaison | 为每个页面生成 Gemini 视觉提示词 |
| `*refine-iteration` | ui-designer-liaison | 基于反馈迭代优化设计提示词 |
| `*log-selection` | ui-designer-liaison | 记录用户选定的设计方案 |
| `*instant` | quick-designer | 一键生成 UI 屏幕（HTML+CSS+预览） |
| `*validate` | quick-designer | 验证并锁定设计变体 |
| `*create` | sm | 创建下一批用户故事 |
| `*review-story` | qa | 审查已实现的故事 |

### MCP 工具（LLM 自动调用，`.mcp.json` 启用后生效）

| 工具 | 描述 |
|------|------|
| `get_project_context` | 读取当前项目阶段、需求、决策、对话历史 |
| `load_agent_persona` | 加载指定阶段的 Agent 人设 |
| `transition_phase` | 切换项目阶段，携带上下文 |
| `generate_deliverable` | 生成文档（PRD/架构/故事等）并保存到 `docs/` |
| `configure_developer_lane` | 配置开发通道行为 |
| `run_story_context_validation` | 验证当前故事上下文的完整性 |

---

## 4. 能力边界

### 能做

- 对话式引导用户梳理需求、定义用户旅程
- 从参考 URL 提取设计 token（颜色、字体、间距）——需 Chrome MCP
- 生成结构化的 PRD、架构文档、用户故事
- 为每个页面生成 Gemini 图片生成提示词（含 CSS token）
- Quick Designer 模式可直接产出 HTML/CSS 代码 + 浏览器预览
- 设计决策追踪 & 设计系统版本管理

### 不能做

- **不直接生成图片** —— 产出的是给 Gemini / v0 / Lovable 的提示词，用户需手动贴到外部工具
- **不是全局工具** —— 每个项目需单独 `init`
- **MCP Server 有已知 bug** —— npm 包缺少 `hooks/phase-transition.js`，MCP 工具层可能不可用；但 Agent Prompt 文件不受影响
- **不额外调用外部 AI API** —— 所有 LLM 交互通过用户已有的 Claude CLI 认证
- **不做后端实现** —— 不写数据库、API、认证等代码

---

## 5. 完整工作流

```
用户提需求
    │
    ▼
analyst → brief.md        需求分析
    │
    ▼
pm → prd.md              产品需求文档
    │
    ▼
ux-expert → spec.md      前端规范
    │
    ▼
ui-designer-liaison       交互式设计对话
    │                     → Gemini 提示词
    ▼
architect → arch.md       技术架构
    │
    ▼
po → sharded docs         验证 & 拆分
    │
    ▼
sm → stories/*.md         用户故事
    │
    ▼
dev ↔ qa                  实现 + 审查（循环）
```

---

## 6. FAQ

**Q: 需要 API Key 吗？**
不需要。AiDesigner 通过用户已有的 Claude CLI 认证运行。

**Q: 设计图怎么生成？**
AiDesigner 产出文本提示词。用户需手动贴到 Google AI Studio (Gemini) 或 v0/Lovable 等工具生成图片。

**Q: MCP Server 起不来怎么办？**
已知 bug——npm 包缺少部分文件。Agent 角色 Prompt 不受影响，Claude 可直接 Read 读取 `aidesigner-core/agents/*.md` 获取人设。
