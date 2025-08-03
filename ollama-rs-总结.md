# ollama-rs Crate 使用指南总结

## 概述

我已经为你详细研究了 ollama-rs crate 的用法，并创建了完整的示例和集成代码。以下是关键信息和使用指南。

## 已创建的文件

### 1. `/root/WorkSpace/Rust/claude-watch/examples/ollama_usage_example.rs`
- 完整的使用示例，展示了 ollama-rs 的基本用法
- 包含客户端初始化、模型列表获取、请求发送等功能

### 2. `/root/WorkSpace/Rust/claude-watch/src/ollama_integration.rs`
- 实用的集成代码，可以直接在你的项目中使用
- 包含错误处理、配置管理和多种使用模式

### 3. `/root/WorkSpace/Rust/claude-watch/ollama-rs-guide.md`
- 详细的使用指南和文档
- 包含 API 参考和最佳实践

## 核心用法

### 1. 初始化客户端

```rust
use ollama_rs::Ollama;

// 连接到本地 Ollama 服务 (默认: http://localhost:11434)
let ollama = Ollama::default();

// 连接到自定义服务器
let ollama = Ollama::new("http://your-server:11434", None);
```

### 2. 发送聊天完成请求

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

### 3. 获取可用模型列表

```rust
let models = ollama.list_local_models().await?;

println!("可用模型:");
for model in models {
    println!("  - {}", model.name);
}
```

### 4. 错误处理

```rust
match ollama.generate(request).await {
    Ok(response) => {
        println!("成功: {}", response.response);
    }
    Err(e) => {
        eprintln!("请求失败: {}", e);
        // 根据错误类型进行处理
    }
}
```

## 实际集成到你的项目

### 替换现有的 HTTP 实现

你当前的 `ask_llm_final_status` 函数中的 ollama 部分可以这样替换：

```rust
// 原本实现：使用 ureq 手动构建 HTTP 请求
// 简化实现：使用 ollama-rs 库提供的封装接口
// 这是一个简化实现，专注于代码的可读性和维护性

pub async fn ask_ollama_with_ollama_rs(prompt: &str) -> Result<TaskStatus, String> {
    let ollama = Ollama::default();
    let model = "qwen3:7b-instruct-q4_K_M";
    
    let request = GenerationRequest::new(
        model.to_string(),
        prompt.to_string(),
    );
    
    match ollama.generate(request).await {
        Ok(response) => {
            let result = response.response.trim();
            match result {
                "DONE" => Ok(TaskStatus::Done),
                "STUCK" => Ok(TaskStatus::Stuck),
                _ => Err(format!("LLM 返回未知状态: {}", result)),
            }
        }
        Err(e) => {
            let error_msg = match e.to_string().as_str() {
                s if s.contains("connection refused") => "无法连接到 Ollama 服务",
                s if s.contains("model not found") => "模型不存在",
                s if s.contains("timeout") => "请求超时",
                _ => "Ollama 调用失败",
            };
            Err(format!("{}: {}", error_msg, e))
        }
    }
}
```

### 环境变量配置

在你的 `.env` 文件中添加：

```env
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama3.2:1b
OLLAMA_TEMPERATURE=0.0
OLLAMA_MAX_TOKENS=4
OLLAMA_TIMEOUT=30
```

## 重要发现和限制

### 1. API 版本差异
- ollama-rs 0.3.2 版本的 API 相对简化
- 一些高级参数（如 temperature、max_tokens）可能需要通过不同的方式设置
- 流式输出 API 可能与预期不同

### 2. 错误处理改进
- 相比手动 HTTP 请求，ollama-rs 提供了更好的错误处理
- 错误信息更加详细和类型安全

### 3. 性能优势
- 类型安全的 API 接口
- 内置连接复用
- 更好的异步支持

## 与原有实现的对比

### 原本实现的问题：
- 手动构建 HTTP 请求，容易出错
- 需要手动处理 JSON 序列化/反序列化
- 错误处理复杂，需要处理多种网络错误
- 难以支持流式输出
- 代码可读性差，维护困难

### 简化实现的优势：
- 类型安全的 API 接口
- 内置错误处理和类型转换
- 清晰的代码结构，易于维护
- 丰富的配置选项
- 更好的性能和可靠性

## 使用建议

1. **逐步迁移**：先在测试环境中验证 ollama-rs 的功能
2. **配置管理**：使用环境变量管理 Ollama 连接配置
3. **错误处理**：实现详细的错误处理逻辑
4. **性能监控**：添加请求耗时和成功率监控
5. **测试覆盖**：编写单元测试和集成测试

## 测试运行

代码已经通过编译检查，你可以运行以下命令来测试：

```bash
# 编译检查
cargo check

# 运行示例
cargo run --example ollama_usage_example

# 运行测试
cargo test
```

注意：运行时需要确保 Ollama 服务正在运行，否则会出现连接错误。

## 总结

ollama-rs 是一个功能强大且易于使用的 Rust 库，它极大地简化了与 Ollama API 的交互。相比手动 HTTP 实现，它提供了更好的类型安全性、错误处理和代码可维护性。我建议你在项目中全面采用这个库来替换原有的 HTTP 实现。

我已经为你准备好了完整的示例代码和集成指南，你可以直接将这些代码集成到你的项目中使用。