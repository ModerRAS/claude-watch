# ollama-rs Crate 使用指南

## 概述

ollama-rs 是一个用于与 Ollama API 交互的 Rust 库。它提供了简单易用的接口来调用本地或远程的 Ollama 服务进行聊天完成。

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
ollama-rs = "0.3.2"
tokio = { version = "1.0", features = ["full"] }
```

## 核心用法

### 1. 初始化客户端

```rust
use ollama_rs::Ollama;

// 连接到本地 Ollama 服务 (默认: http://localhost:11434)
let ollama = Ollama::default();

// 连接到自定义服务器
let ollama = Ollama::new("http://your-server:11434", None);
```

### 2. 发送基本聊天完成请求

```rust
use ollama_rs::generation::completion::request::GenerationRequest;

// 创建请求
let request = GenerationRequest::new(
    "llama3.2:1b".to_string(),  // 模型名称
    "你好，请介绍一下 Rust 语言".to_string(),  // 提示词
);

// 发送请求并获取响应
let response = ollama.generate(request).await?;
println!("响应: {}", response.response);
```

### 3. 流式响应

```rust
let request = GenerationRequest::new(
    "llama3.2:1b".to_string(),
    "请写一个关于编程的长段落".to_string(),
)
.stream(true);  // 启用流式输出

let mut stream = ollama.generate_stream(request).await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(chunk) => {
            print!("{}", chunk.response);
        }
        Err(e) => {
            eprintln!("错误: {}", e);
            break;
        }
    }
}
```

### 4. 模型参数配置

```rust
let request = GenerationRequest::new(
    "llama3.2:1b".to_string(),
    "请回答我的问题".to_string(),
)
.temperature(0.7)              // 控制随机性 (0.0-1.0)
.top_p(0.9)                    // 核采样
.top_k(40)                     // Top-k 采样
.max_tokens(200)               // 最大令牌数
.repeat_penalty(1.1)           // 重复惩罚
.presence_penalty(0.0)         // 存在惩罚
.frequency_penalty(0.0)        // 频率惩罚
.seed(42)                      // 随机种子
.num_ctx(2048)                 // 上下文窗口大小
.num_predict(250)              // 最大预测令牌数
.stop(vec!["\n\n".to_string()]); // 停止序列
```

### 5. 获取可用模型列表

```rust
use ollama_rs::models::ModelsRequest;

let models_request = ModelsRequest::default();
let models = ollama.list_models(models_request).await?;

for model in &models.models {
    println!("模型: {}, 大小: {:.2} GB", model.name, model.size / 1024.0 / 1024.0 / 1024.0);
}
```

### 6. 错误处理

```rust
match ollama.generate(request).await {
    Ok(response) => {
        println!("成功: {}", response.response);
    }
    Err(e) => {
        eprintln!("请求失败: {}", e);
        // 处理特定错误类型
        match e {
            ollama_rs::error::Error::Request(_) => {
                println!("网络请求错误");
            }
            ollama_rs::error::Error::Api(_) => {
                println!("API 错误");
            }
            _ => {
                println!("其他错误");
            }
        }
    }
}
```

## 集成到现有项目

### 替换现有的 HTTP 实现

如果你当前的代码使用 `ureq` 直接调用 Ollama API，可以这样替换：

```rust
// 原本实现：使用 ureq 直接调用 API
// 这是一个简化实现，移除了手动 HTTP 请求构建

pub async fn ask_ollama_with_ollama_rs(prompt: &str, model: &str) -> Result<String, Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    
    let request = GenerationRequest::new(
        model.to_string(),
        prompt.to_string(),
    )
    .temperature(0.0)
    .max_tokens(4)
    .stream(false);
    
    let response = ollama.generate(request).await?;
    Ok(response.response)
}
```

### 在你的项目中使用

将现有的 `ask_llm_final_status` 函数中的 ollama 部分替换：

```rust
// 原本实现：使用 ureq 手动构建 HTTP 请求
// 简化实现：使用 ollama-rs 库提供的封装接口
// 这是一个简化实现，专注于代码的可读性和维护性

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};

async fn ask_ollama_with_ollama_rs(prompt: &str) -> Result<TaskStatus, String> {
    let ollama = Ollama::default();
    let model = "qwen3:7b-instruct-q4_K_M";
    
    let request = GenerationRequest::new(
        model.to_string(),
        prompt.to_string(),
    )
    .temperature(0.0)
    .max_tokens(4)
    .stream(false);
    
    match ollama.generate(request).await {
        Ok(response) => {
            let result = response.response.trim();
            match result {
                "DONE" => Ok(TaskStatus::Done),
                "STUCK" => Ok(TaskStatus::Stuck),
                _ => Err(format!("LLM 返回未知状态: {}", result)),
            }
        }
        Err(e) => Err(format!("Ollama 调用失败: {}", e)),
    }
}
```

## 高级用法

### 1. 异步上下文

```rust
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 你的异步代码
    Ok(())
}
```

### 2. 对话历史管理

```rust
struct ChatHistory {
    messages: Vec<(String, String)>, // (role, content)
}

impl ChatHistory {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
    
    fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push((role.to_string(), content.to_string()));
    }
    
    fn build_context(&self) -> String {
        self.messages
            .iter()
            .map(|(role, content)| format!("{}: {}", role, content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// 使用示例
let mut history = ChatHistory::new();
history.add_message("user", "你好");
history.add_message("assistant", "你好！有什么我可以帮助的吗？");

let context = history.build_context();
let request = GenerationRequest::new(
    "llama3.2:1b".to_string(),
    format!("基于以下对话历史回答问题：\n\n{}\n\n问题：{}", context, "我是谁？"),
);
```

### 3. 批量处理

```rust
use futures::future::try_join_all;

let prompts = vec![
    "什么是 Rust？",
    "什么是人工智能？",
    "什么是机器学习？",
];

let requests: Vec<_> = prompts
    .into_iter()
    .map(|prompt| {
        let ollama = ollama.clone();
        let request = GenerationRequest::new(
            "llama3.2:1b".to_string(),
            prompt.to_string(),
        );
        
        async move {
            ollama.generate(request).await
        }
    })
    .collect();

let responses = try_join_all(requests).await?;

for (i, response) in responses.iter().enumerate() {
    println!("问题 {}: {}", i + 1, response.response);
}
```

## 性能优化建议

1. **连接复用**：重用 `Ollama` 实例，避免重复创建
2. **并发请求**：使用 `tokio` 的并发特性处理多个请求
3. **流式输出**：对于长文本，使用流式输出改善用户体验
4. **参数调优**：根据任务需求调整 `temperature`、`max_tokens` 等参数

## 常见问题解决

1. **连接失败**：检查 Ollama 服务是否运行在默认端口 11434
2. **模型不存在**：使用 `ollama list` 查看可用模型，或使用 `ollama pull` 下载新模型
3. **响应超时**：增加 `max_tokens` 或检查网络连接
4. **内存不足**：使用较小的模型或减少上下文窗口大小

## 与现有代码的对比

### 原本实现的问题：
- 手动构建 HTTP 请求，容易出错
- 需要手动处理 JSON 序列化/反序列化
- 错误处理复杂，需要处理多种网络错误
- 难以支持流式输出
- 代码可读性差，维护困难

### 简化实现的优势：
- 类型安全的 API 接口
- 内置错误处理和类型转换
- 轻松支持流式输出
- 清晰的代码结构，易于维护
- 丰富的配置选项
- 更好的性能和可靠性

## 总结

ollama-rs 提供了一个完整的、类型安全的 Rust 接口来与 Ollama API 交互。相比直接使用 HTTP 客户端，它提供了更好的开发体验、错误处理和功能支持。建议在项目中全面采用这个库来替换原有的 HTTP 实现。