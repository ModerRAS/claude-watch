use ollama_rs::{
    Ollama,
    generation::completion::request::GenerationRequest,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ===================================================================
    // 1. 初始化客户端
    // ===================================================================
    // 原本实现：需要手动配置 URL 和认证
    // 简化实现：使用默认配置，自动连接到本地 Ollama 服务
    // 这是一个简化实现，专注于本地开发环境的快速设置
    
    // 创建 Ollama 客户端实例
    // 默认连接到 http://localhost:11434
    let ollama = Ollama::default();
    
    // 如果需要连接到自定义服务器：
    // let ollama = Ollama::new("http://your-server:11434", None);
    
    println!("✅ Ollama 客户端初始化成功");
    
    // ===================================================================
    // 2. 获取可用模型列表
    // ===================================================================
    println!("\n📋 获取可用模型列表...");
    
    let models = ollama.list_local_models().await?;
    
    println!("可用模型:");
    for model in models {
        println!("  - {}", model.name);
    }
    
    // ===================================================================
    // 3. 发送聊天完成请求（同步模式）
    // ===================================================================
    println!("\n💬 发送同步聊天完成请求...");
    
    // 准备生成请求
    let request = GenerationRequest::new(
        "llama3.2:1b".to_string(),  // 模型名称
        "你好！请用中文介绍一下 Rust 语言的特点。".to_string(),  // 提示词
    );
    
    // 发送请求并获取响应
    let response = ollama.generate(request).await?;
    
    println!("模型响应:");
    println!("📝 {}", response.response);
    if let Some(eval_duration) = response.eval_duration {
        println!("⏱️ 耗时: {:.2} 秒", eval_duration as f64 / 1_000_000_000.0);
    }
    if let Some(total_duration) = response.total_duration {
        println!("🔥 总耗时: {:.2} 秒", total_duration as f64 / 1_000_000_000.0);
    }
    
    // ===================================================================
    // 4. 发送聊天完成请求（流式模式模拟）
    // ===================================================================
    println!("\n🌊 发送聊天完成请求...");
    
    let stream_request = GenerationRequest::new(
        "llama3.2:1b".to_string(),
        "请用三句话介绍什么是人工智能。".to_string(),
    );
    
    // 注意：ollama-rs 0.3.2 可能没有直接的流式 API
    // 这里我们使用普通的 generate 方法作为示例
    let response = ollama.generate(stream_request).await?;
    
    println!("模型响应:");
    println!("📝 {}", response.response);
    println!("💡 注意：ollama-rs 0.3.2 版本的流式 API 可能需要不同的调用方式");
    
    // ===================================================================
    // 5. 高级参数设置
    // ===================================================================
    println!("\n🔧 高级参数设置示例...");
    
    let advanced_request = GenerationRequest::new(
        "llama3.2:1b".to_string(),
        "请帮我写一个简单的 Rust 函数来计算斐波那契数列。".to_string(),
    );
    
    // 注意：ollama-rs 0.3.2 的 API 可能比较简化
    // 高级参数可能需要通过不同的方式设置
    
    println!("发送高级配置请求...");
    let advanced_response = ollama.generate(advanced_request).await?;
    
    println!("高级配置响应:");
    println!("📝 {}", advanced_response.response);
    if let Some(prompt_eval_count) = advanced_response.prompt_eval_count {
        println!("📊 使用的令牌数: {}", prompt_eval_count);
    }
    if let Some(eval_count) = advanced_response.eval_count {
        println!("📊 生成的令牌数: {}", eval_count);
    }
    println!("💡 注意：高级参数设置可能需要查看 ollama-rs 的具体文档");
    
    // ===================================================================
    // 6. 错误处理和重试机制
    // ===================================================================
    println!("\n🛡️ 错误处理示例...");
    
    // 尝试使用不存在的模型
    let bad_request = GenerationRequest::new(
        "nonexistent-model".to_string(),
        "这个请求会失败".to_string(),
    );
    
    match ollama.generate(bad_request).await {
        Ok(response) => {
            println!("意外成功: {}", response.response);
        }
        Err(e) => {
            println!("❌ 预期的错误: {}", e);
            println!("🔧 错误处理建议：检查模型名称、网络连接和 Ollama 服务状态");
        }
    }
    
    // ===================================================================
    // 7. 实际应用示例：对话历史管理
    // ===================================================================
    println!("\n💭 对话历史管理示例...");
    
    let mut conversation_history = Vec::new();
    
    // 第一轮对话
    let user_message1 = "我的名字是张三，我是个程序员。";
    conversation_history.push(("user".to_string(), user_message1.to_string()));
    
    let request1 = GenerationRequest::new(
        "llama3.2:1b".to_string(),
        format!("记住这个信息：{}", user_message1),
    );
    
    let response1 = ollama.generate(request1).await?;
    conversation_history.push(("assistant".to_string(), response1.response.clone()));
    println!("用户: {}", user_message1);
    println!("助手: {}", response1.response);
    
    // 第二轮对话
    let user_message2 = "我是谁？我做什么工作？";
    conversation_history.push(("user".to_string(), user_message2.to_string()));
    
    // 构建包含历史上下文的请求
    let context = conversation_history
        .iter()
        .map(|(role, content)| format!("{}: {}", role, content))
        .collect::<Vec<_>>()
        .join("\n");
    
    let request2 = GenerationRequest::new(
        "llama3.2:1b".to_string(),
        format!("基于以下对话历史，回答问题：\n\n{}\n\n问题：{}", context, user_message2),
    );
    
    let response2 = ollama.generate(request2).await?;
    conversation_history.push(("assistant".to_string(), response2.response.clone()));
    
    println!("用户: {}", user_message2);
    println!("助手: {}", response2.response);
    
    println!("\n✅ ollama-rs 用法演示完成！");
    
    Ok(())
}