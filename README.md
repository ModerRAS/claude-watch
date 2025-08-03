# claude-watch

一个极简守护进程，基于**读秒检测**监控 Claude Code，  
读秒停止时**立即**调用 LLM 判断最终状态：

- **有读秒** → Claude Code 正常工作，继续监控
- **读秒停止** → **立即**调用 LLM 判断是 DONE 还是 STUCK
- DONE → 自动退出，留给后续 CI/脚本继续  
- STUCK → 自动 `Retry`；重试 10 次后 `/compact`

## 核心设计思路

### 读秒检测策略（关键创新）

**原本实现**：复杂的启发式规则和频繁的 LLM 调用
**简化实现**：基于读秒检测的智能触发机制

#### 工作原理：
1. **主要监控**：通过正则表达式检测各种计时器格式
   - `⏱ 00:42`、`⌛ 00:42`（常见计时器）
   - `计时: 42秒`、`时间: 42秒`（中文计时器）
   - `[42%]`、`42% 完成`（进度指示器）
   - `正在处理`、`Working on`（活动状态）

2. **触发逻辑**：
   - ✅ **有读秒** → Claude Code 正在工作，继续监控
   - ⏸️ **读秒停止** → **立即**调用 LLM 判断状态
   - 🤖 **LLM 判断** → 返回 DONE 或 STUCK
   - 🔄 **重试机制** → STUCK 时自动重试或发送 `/compact`

#### 核心优势：
- **极低 LLM 调用频率**：可能几分钟甚至几小时才调用一次
- **零延迟监控**：读秒检测是纳秒级的正则匹配
- **高准确性**：LLM 只在关键时刻判断，避免误判
- **资源友好**：大幅减少网络请求和计算资源消耗

## 核心函数说明

### 读秒检测函数
- `has_timer_running()` - 核心读秒检测器，支持多种计时器格式

### LLM 判断函数
- `ask_llm_final_status()` - 仅在读秒停止时调用，判断最终状态
- `simple_heuristic_check()` - LLM 不可用时的备用方案

### 主要逻辑流程
1. **持续监控**：每 `INTERVAL` 秒检查一次 tmux pane
2. **读秒检测**：使用 `has_timer_running()` 判断工作状态
3. **即时判断**：读秒停止时**立即**调用 LLM，无等待
4. **结果处理**：根据 LLM 结果退出或重试

## 快速开始

### 1. 克隆并编译
```bash
git clone <url> claude-watch && cd claude-watch
cargo build --release
```

### 2. 配置环境变量
```bash
cp .env.example .env
# 编辑 .env，至少配置 PANE 和 LLM_BACKEND
```

### 3. 选择运行模式

#### 模式 A：使用 LLM（最准确）
```bash
# .env 中设置
LLM_BACKEND=ollama
OLLAMA_URL=http://localhost:11434/api/generate
```
需要先启动 Ollama 并拉取模型。

#### 模式 B：纯启发式（零依赖）
```bash
# .env 中设置
LLM_BACKEND=none
```
无需任何外部服务，开箱即用。

### 4. 运行
```bash
./target/release/claude-watch
```

## 使用方法

1. 在 tmux 里启动 Claude Code。  
2. 把 pane id 写进 `.env`（如 `PANE=%0`）。  
3. 运行监控程序，观察实时日志输出。  
4. 任务完成后程序自动退出。

## 配置说明

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `PANE` | `%0` | 要监控的 tmux pane ID |
| `INTERVAL` | `5` | 检查间隔（秒） |
| `MAX_RETRY` | `10` | 最大重试次数 |
| `LLM_BACKEND` | `ollama` | LLM 后端：`ollama`、`openrouter`、`none` |
| `OLLAMA_URL` | `http://localhost:11434/api/generate` | Ollama API 地址 |
| `OPENROUTER_KEY` | - | OpenRouter API 密钥 |
| `OPENROUTER_MODEL` | `qwen/qwen3-7b-instruct` | OpenRouter 模型 |

## 故障处理

### LLM 调用失败
程序会自动降级到启发式规则，并显示警告信息。

### 网络问题
如果 LLM_BACKEND 不是 `none`，网络问题会触发降级策略。

### tmux 连接问题
确保 tmux 正在运行且指定的 pane 存在。