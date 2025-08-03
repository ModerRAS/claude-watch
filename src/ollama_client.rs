use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use serde_json::json;

/// 原本实现：使用 ureq 手动构建 HTTP 请求发送到 Ollama API
/// 简化实现：使用 ollama-rs 库提供的高级 API 接口
/// 这是一个简化实现，移除了手动 JSON 构建和 HTTP 请求处理
pub async fn ask_ollama_with_ollama_rs(
    prompt_text: &str, 
    model: &str
) -> Result<String, String> {
    // 初始化 Ollama 客户端（使用默认配置连接到本地服务）
    let ollama = Ollama::default();
    
    // 构建生成请求
    let request = GenerationRequest::new(
        model.to_string(),
        prompt_text.to_string(),
    );
    
    // 发送请求并处理响应
    match ollama.generate(request).await {
        Ok(response) => {
            Ok(response.response)
        }
        Err(e) => {
            Err(format!("Ollama 调用失败: {}", e))
        }
    }
}