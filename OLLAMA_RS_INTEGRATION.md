# Ollama-rs 集成说明

这个文档说明了如何在 claude-watch 项目中集成和使用 ollama-rs 库。

## 依赖添加

在 `Cargo.toml` 中添加了以下依赖：

```toml
[dependencies]
ollama-rs = "0.3.2"
tokio = { version = "1.0", features = ["full"] }
```

## 实现细节

### 1. 异步函数封装

我们创建了一个异步函数来处理 Ollama API 调用：

```rust
async fn ask_ollama_with_ollama_rs(prompt_text: &str, model: &str) -> Result<String, String> {
    // 初始化 Ollama 客户端
    let ollama = ollama_rs::Ollama::default();
    
    // 构建生成请求
    let request = ollama_rs::generation::completion::request::GenerationRequest::new(
        model.to_string(),
        prompt_text.to_string(),
    );
    
    // 发送请求并处理响应
    match ollama.generate(request).await {
        Ok(response) => Ok(response.response),
        Err(e) => Err(format!("Ollama 调用失败: {}", e))
    }
}
```

### 2. 同步调用桥接

由于主程序是同步的，我们使用 tokio 运行时来桥接异步调用：

```rust
// 在同步函数中调用异步函数
let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建运行时失败: {}", e))?;
match rt.block_on(ask_ollama_with_ollama_rs(&full_prompt, model)) {
    Ok(response) => {
        let response = response.trim();
        match response {
            "DONE" => Ok(TaskStatus::Done),
            "STUCK" => Ok(TaskStatus::Stuck),
            _ => Err(format!("LLM 返回未知状态: {}", response)),
        }
    }
    Err(e) => Err(e),
}
```

## 优势

使用 ollama-rs 相比手动 HTTP 请求的优势：

1. **类型安全**：编译时检查参数和响应类型
2. **错误处理**：提供详细的错误信息和类型化错误
3. **代码简洁**：移除手动 JSON 处理和 HTTP 请求构建
4. **更好的维护性**：使用标准库 API，减少自定义代码

## 使用说明

1. 确保 Ollama 服务正在运行
2. 拉取所需模型：`ollama pull qwen3:7b-instruct-q4_K_M`
3. 设置环境变量：
   ```bash
   LLM_BACKEND=ollama
   ```
4. 运行程序即可自动使用 ollama-rs 进行 API 调用

## 注意事项

1. ollama-rs 0.3.2 版本的 API 相对简化，某些高级参数可能需要额外配置
2. 需要 tokio 运行时来处理异步操作
3. 错误处理已经优化，提供更友好的错误信息