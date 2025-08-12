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

### 配置文件系统

现在支持通过 YAML 配置文件进行配置管理，推荐使用配置文件而不是环境变量。

#### 创建配置文件

```bash
# 复制示例配置文件
cp config.example.yaml config.yaml
# 编辑配置文件
vim config.yaml
```

#### 配置文件结构

```yaml
# LLM Backend Configuration
llm:
  # Backend type: ollama, openai, openrouter, none
  backend: "ollama"
  
  # Ollama configuration (used when backend is "ollama")
  ollama:
    url: "http://localhost:11434"
    model: "qwen2.5:3b"
  
  # OpenAI configuration (used when backend is "openai")
  openai:
    api_key: "sk-..."  # Your OpenAI API key
    api_base: "https://api.openai.com/v1"  # OpenAI API base URL
    model: "gpt-4o"
  
  # OpenRouter configuration (used when backend is "openrouter")
  openrouter:
    api_key: "sk-or-..."  # Your OpenRouter API key
    model: "qwen/qwen-2.5-7b-instruct"

# Monitoring Configuration
monitoring:
  # Check interval in seconds
  interval: 5
  
  # Stuck timeout in seconds
  stuck_sec: 60
  
  # Maximum retry attempts
  max_retry: 10

# Tmux Configuration
tmux:
  # Tmux pane ID (e.g., %0 or mysess:1.0)
  pane: "%0"
```

### 命令行参数

| 参数 | 短参数 | 默认值 | 说明 |
|------|--------|--------|------|
| `--config` | `-c` | `config.yaml` | 配置文件路径 |
| `--pane` | `-p` | 从配置文件读取 | 要监控的 tmux pane ID |
| `--backend` | `-b` | 从配置文件读取 | LLM 后端：`ollama`、`openai`、`openrouter`、`none` |
| `--interval` | `-i` | 从配置文件读取 | 检查间隔（秒） |
| `--stuck-sec` | `-s` | 从配置文件读取 | 无活动多久算卡住（秒） |
| `--max-retry` | `-m` | 从配置文件读取 | 最大重试次数 |

### 环境变量（兼容性支持）

为保持向后兼容性，仍然支持环境变量配置，但推荐使用配置文件。

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `OLLAMA_URL` | `http://localhost:11434` | Ollama 服务器地址 |
| `OPENROUTER_KEY` | - | OpenRouter API 密钥 |
| `OPENROUTER_MODEL` | `qwen/qwen-2.5-7b-instruct` | OpenRouter 模型（可选） |
| `OPENAI_API_KEY` | - | OpenAI API 密钥 |
| `OPENAI_API_BASE` | `https://api.openai.com/v1` | OpenAI API 基础URL |

### 使用示例

```bash
# 基本用法（使用默认配置文件 config.yaml）
./claude-watch

# 指定自定义配置文件
./claude-watch --config my-config.yaml

# 使用 OpenAI 后端
./claude-watch --config config.yaml --backend openai

# 使用 OpenRouter 后端
./claude-watch --config config.yaml --backend openrouter

# 自定义检查间隔和卡住判定时间
./claude-watch --config config.yaml --interval 10 --stuck-sec 120

# 增加重试次数
./claude-watch --config config.yaml --max-retry 20

# 使用短参数形式
./claude-watch -c config.yaml -b ollama -i 5 -s 60 -m 10
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
2. 在配置文件中设置：
   ```yaml
   llm:
     backend: "openrouter"
     openrouter:
       api_key: "sk-or-v1-xxx"
       model: "qwen/qwen-2.5-7b-instruct"  # 可选，默认模型
   ```
3. 或者设置环境变量：
   ```bash
   OPENROUTER_KEY=sk-or-v1-xxx
   OPENROUTER_MODEL=qwen/qwen-2.5-7b-instruct  # 可选，默认模型
   ```
4. 使用命令行参数：`--backend openrouter`

## OpenAI 支持说明

要使用 OpenAI 或兼容的 API 服务器：
1. 在配置文件中设置：
   ```yaml
   llm:
     backend: "openai"
     openai:
       api_key: "sk-xxx"
       api_base: "https://api.openai.com/v1"  # 或你的自定义 API 服务器地址
       model: "gpt-4o"
   ```
2. 对于自定义 API 服务器（兼容 OpenAI API 格式）：
   ```yaml
   llm:
     backend: "openai"
     openai:
       api_key: "your-api-key"
       api_base: "https://your-api-server.com/v1"
       model: "your-model-name"
   ```
3. 或者使用环境变量：
   ```bash
   OPENAI_API_KEY=sk-xxx
   OPENAI_API_BASE=https://api.openai.com/v1
   ```
4. 使用命令行参数：`--backend openai`

注意：OpenAI 后端使用 [openai-api-rs](https://github.com/dongri/openai-api-rs) 库，支持所有兼容 OpenAI API 格式的服务。

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