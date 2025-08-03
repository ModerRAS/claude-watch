# claude-watch

一个极简守护进程，基于**Claude Code 活动检测**监控 Claude Code，  
检测到无活动时调用 LLM 判断最终状态：

- **有活动** → Claude Code 正常工作，继续监控
- **无活动** → 等待配置的时间后调用 LLM 判断是 DONE 还是 STUCK
- DONE → 进入守护模式，监控新任务开始
- STUCK → 自动 `Retry`；重试配置次数后 `/compact`

## 核心设计思路

### Claude Code 活动检测策略（关键创新）

**原本实现**：简单的画面变化检测
**简化实现**：基于 Claude Code 特定输出模式的智能检测

#### 工作原理：
1. **主要监控**：检测 Claude Code 特定的活动输出模式
   - 类似 `104s` 的时间格式（数字+s）
   - `tokens` 计数信息
   - `Processing` 状态或 `↑↓` 传输指示器

2. **触发逻辑**：
   - ✅ **有活动** → Claude Code 正在工作，继续监控
   - ⏸️ **无活动** → 等待 `STUCK_SEC` 秒后调用 LLM 判断状态
   - 🤖 **LLM 判断** → 返回 DONE 或 STUCK
   - 🔄 **重试机制** → STUCK 时自动重试或发送 `/compact`

#### 核心优势：
- **极低 LLM 调用频率**：只有在 Claude Code 停止工作时才调用
- **高准确性**：专门针对 Claude Code 的输出模式设计
- **低误判率**：基于特定的输出格式而不是简单的画面变化
- **资源友好**：大幅减少网络请求和计算资源消耗

## 核心函数说明

### Claude Code 活动检测函数
- `is_claude_active()` - 核心活动检测器，检测 Claude Code 特定输出模式

### LLM 判断函数
- `ask_llm_final_status()` - 仅在无活动时调用，判断最终状态
- `simple_heuristic_check()` - LLM 不可用时的备用方案

### 主要逻辑流程
1. **持续监控**：每 `INTERVAL` 秒检查一次 tmux pane
2. **活动检测**：使用 `is_claude_active()` 判断 Claude Code 工作状态
3. **超时判断**：无活动超过 `STUCK_SEC` 秒后调用 LLM
4. **结果处理**：根据 LLM 结果退出或重试

## 快速开始

### 1. 克隆并编译
```bash
git clone <url> claude-watch && cd claude-watch
cargo build --release
```

### 2. 配置环境变量（可选）
```bash
cp .env.example .env
# 编辑 .env，设置 LLM 服务地址和 API 密钥
```

### 3. 选择运行模式

#### 模式 A：使用 Ollama（推荐）
```bash
./target/release/claude-watch --pane %0 --backend ollama
```
需要先启动 Ollama 并拉取模型。

#### 模式 B：使用 OpenRouter
```bash
./target/release/claude-watch --pane %0 --backend openrouter
```

#### 模式 C：纯启发式（零依赖）
```bash
./target/release/claude-watch --pane %0 --backend none
```
无需任何外部服务，开箱即用。

### 4. 运行
```bash
# 基本用法
./target/release/claude-watch --pane %0

# 自定义参数
./target/release/claude-watch --pane %0 --backend ollama --interval 10 --stuck-sec 120

# 查看帮助
./target/release/claude-watch --help
```

## 使用方法

1. 在 tmux 里启动 Claude Code。  
2. 运行监控程序，使用 `--pane` 参数指定 pane ID。  
3. 观察实时日志输出，程序会自动检测和处理状态。  
4. 任务完成后程序进入守护模式，等待新任务开始。

## 配置说明

### 命令行参数

| 参数 | 短参数 | 默认值 | 说明 |
|------|--------|--------|------|
| `--pane` | `-p` | （必需） | 要监控的 tmux pane ID |
| `--backend` | `-b` | `ollama` | LLM 后端：`ollama`、`openrouter`、`none` |
| `--interval` | `-i` | `5` | 检查间隔（秒） |
| `--stuck-sec` | `-s` | `60` | 无活动多久算卡住（秒） |
| `--max-retry` | `-m` | `10` | 最大重试次数 |

### 环境变量

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `OLLAMA_URL` | `http://localhost:11434` | Ollama 服务器地址 |
| `OPENROUTER_KEY` | - | OpenRouter API 密钥 |
| `OPENROUTER_MODEL` | `qwen/qwen-2.5-7b-instruct` | OpenRouter 模型（可选） |

### 使用示例

```bash
# 基本用法
./claude-watch --pane %0

# 使用 OpenRouter 后端
./claude-watch --pane %0 --backend openrouter

# 自定义检查间隔和卡住判定时间
./claude-watch --pane %0 --interval 10 --stuck-sec 120

# 增加重试次数
./claude-watch --pane %0 --max-retry 20

# 使用短参数形式
./claude-watch -p %0 -b ollama -i 5 -s 60 -m 10
```

## Ollama 支持说明

本项目使用 [ollama-rs](https://crates.io/crates/ollama-rs) 库来与 Ollama 服务进行交互：

- 使用类型安全的 API 调用替代手动 HTTP 请求
- 自动处理 JSON 序列化/反序列化
- 提供更好的错误处理和诊断信息
- 支持连接到本地或远程 Ollama 实例

要使用 Ollama 后端：
1. 安装并启动 [Ollama](https://ollama.com/)
2. 拉取所需模型：`ollama pull qwen2.5:3b`
3. 使用命令行参数：`--backend ollama`
4. （可选）设置环境变量 `OLLAMA_URL` 指定自定义服务器地址

## OpenRouter 支持说明

要使用 OpenRouter 后端：
1. 在 [OpenRouter](https://openrouter.ai/) 获取 API 密钥
2. 设置环境变量：
   ```bash
   OPENROUTER_KEY=sk-or-v1-xxx
   OPENROUTER_MODEL=qwen/qwen-2.5-7b-instruct  # 可选，默认模型
   ```
3. 使用命令行参数：`--backend openrouter`

## 故障处理

### LLM 调用失败
程序会自动降级到启发式规则，并显示警告信息。

### 网络问题
如果后端不是 `none`，网络问题会触发降级策略。

### tmux 连接问题
确保 tmux 正在运行且指定的 pane 存在。使用 `tmux list-panes` 查看可用的 pane ID。

### 常见问题

**Q: 如何查看可用的 tmux pane ID？**
```bash
tmux list-panes -a
```

**Q: 程序启动后立即退出怎么办？**
检查指定的 pane ID 是否存在，以及 Claude Code 是否在对应的 pane 中运行。

**Q: 如何自定义 Ollama 服务器地址？**
设置环境变量 `OLLAMA_URL`，例如：
```bash
export OLLAMA_URL=http://192.168.1.100:11434
./claude-watch --pane %0 --backend ollama
```