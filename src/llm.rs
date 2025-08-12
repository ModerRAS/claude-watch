use crate::config::Config;
use serde_json::{json, Value};
use tokio;

/// 任务状态枚举
#[derive(Debug, PartialEq)]
pub enum TaskStatus {
    Done,
    Stuck,
}

/// 解析 Ollama URL 为主机和端口
fn parse_ollama_url(url: &str) -> (String, u16) {
    // 移除协议前缀
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    
    // 分割主机和端口
    let parts: Vec<&str> = url.split(':').collect();
    match parts.as_slice() {
        [host, port_str] => {
            let port = port_str.parse::<u16>().unwrap_or(11434);
            (host.to_string(), port)
        }
        [host] => {
            // 默认端口 11434
            (host.to_string(), 11434)
        }
        _ => {
            // 默认值
            ("localhost".to_string(), 11434)
        }
    }
}

/// 使用 ollama-rs 库调用 Ollama API
async fn ask_ollama_with_ollama_rs(prompt_text: &str, model: &str, url: &str) -> Result<String, String> {
    // 解析 URL 获取主机和端口
    let (host, port) = parse_ollama_url(url);
    
    // 初始化 Ollama 客户端（支持自定义服务器地址和端口）
    let ollama = ollama_rs::Ollama::new(&host, port);
    
    // 设置模型选项以提高稳定性和一致性
    let options = ollama_rs::models::ModelOptions::default()
        .temperature(0.0)  // 确保确定性输出
        .num_predict(4);   // 限制输出长度
    
    // 构建生成请求
    let request = ollama_rs::generation::completion::request::GenerationRequest::new(
        model.to_string(),
        prompt_text.to_string(),
    ).options(options);
    
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

/// 使用 openai-api-rs 库调用 OpenAI 兼容的 API
async fn ask_openai(prompt_text: &str, config: &crate::config::OpenAiConfig) -> Result<String, String> {
    use openai_api_rs::v1::api::OpenAIClient;
    use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
    
    // 检查 API key 是否为空
    if config.api_key.is_empty() {
        return Err("OpenAI API key 未设置".to_string());
    }
    
    // 创建 OpenAI 客户端
    let mut client = OpenAIClient::builder()
        .with_endpoint(&config.api_base)
        .with_api_key(config.api_key.clone())
        .build()
        .map_err(|e| format!("创建 OpenAI 客户端失败: {}", e))?;
    
    // 创建聊天完成请求
    let request = ChatCompletionRequest::new(
        config.model.clone(),
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text("你是 Claude Code 状态判别器。用户会粘贴一段 tmux pane 文本。\n\n1. 忽略所有以 `─`, `│`, `╭`, `╰`, `?`, `>`, `ctrl+r`, `… +N lines` 等边框/提示符开头的行。\n2. 找到 Claude 主动输出的最后一句话（通常是 ● 开头或普通文本）。\n3. 判断这句话：\n   - 如果它表示\"完成\"\"成功\"\"已推送\"\"下一步可继续\" → DONE  \n   - 如果它是未完结的预告（如\"让我…\"\"接下来我将…\"\"正在…\"）→ STUCK  \n\n只返回 DONE 或 STUCK。".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(prompt_text.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
    ).max_tokens(4).temperature(0.0);
    
    // 发送请求
    match client.chat_completion(request).await {
        Ok(result) => {
            if let Some(choice) = result.choices.first() {
                if let Some(content) = &choice.message.content {
                    Ok(content.to_string())
                } else {
                    Err("返回内容为空".to_string())
                }
            } else {
                Err("没有返回结果".to_string())
            }
        }
        Err(e) => Err(format!("OpenAI 调用失败: {}", e)),
    }
}

/// 简化的启发式检查（仅在 LLM 不可用时使用）
fn simple_heuristic_check(text: &str) -> TaskStatus {
    // 检查明显的完成标志
    let done_patterns = [
        "✅ All checks passed",
        "Build completed successfully", 
        "Task finished",
        "All tasks completed",
        "任务完成",
        "搞定",
        "完成了",
        "Finished",
        "Completed",
        "Done",
        "✅",
    ];
    
    if done_patterns.iter().any(|&pattern| text.contains(pattern)) {
        return TaskStatus::Done;
    }
    
    // 检查明显的错误标志
    let error_patterns = [
        "Error:",
        "error:", 
        "Failed",
        "failed",
        "panic!",
        "stack trace",
        "出错",
        "失败",
        "错误",
        "卡住",
        "stuck",
        "timeout",
        "超时",
        "无响应",
    ];
    
    if error_patterns.iter().any(|&pattern| text.contains(pattern)) {
        return TaskStatus::Stuck;
    }
    
    // 默认认为卡住（因为画面已经停止变化了）
    TaskStatus::Stuck
}

/// 使用 LLM 判断 Claude Code 最终状态
/// 
/// 这是最关键的状态判断函数，仅在画面长时间无变化时调用
/// 根据配置的 LLM 后端类型进行判断：
/// - "ollama": 使用 Ollama 服务
/// - "openai": 使用 OpenAI 或兼容服务
/// - "openrouter": 使用 OpenRouter 服务
/// - "none": 使用简单的启发式判断
pub fn ask_llm_final_status(text: &str, backend: &str, config: &Config) -> Result<TaskStatus, String> {
    if backend == "none" {
        // 如果禁用 LLM，使用简单的启发式判断
        return Ok(simple_heuristic_check(text));
    }
    
    let prompt = include_str!("../prompt_final.md");
    let full_prompt = format!("{}\n\n{}", prompt, text);

    match backend.as_ref() {
        "ollama" => {
            // 使用 tokio 运行时来执行异步函数
            let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建运行时失败: {}", e))?;
            let model = config.llm.ollama.as_ref().map(|o| o.model.clone()).unwrap_or("qwen2.5:3b".to_string());
            let url = config.llm.ollama.as_ref().map(|o| o.url.clone()).unwrap_or("http://localhost:11434".to_string());
            
            match rt.block_on(ask_ollama_with_ollama_rs(&full_prompt, &model, &url)) {
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
        }
        "openai" => {
            // 使用 tokio 运行时来执行异步函数
            let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建运行时失败: {}", e))?;
            
            if let Some(openai_config) = &config.llm.openai {
                match rt.block_on(ask_openai(&full_prompt, &openai_config)) {
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
            } else {
                Err("OpenAI 配置未找到".to_string())
            }
        }
        "openrouter" => {
            let url = "https://openrouter.ai/api/v1/chat/completions";
            let model = config.llm.openrouter.as_ref().map(|o| o.model.clone()).unwrap_or("qwen/qwen-2.5-7b-instruct".to_string());
            
            if let Some(openrouter_config) = &config.llm.openrouter {
                let body = json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": prompt},
                        {"role": "user", "content": text}
                    ],
                    "max_tokens": 4,
                    "temperature": 0.0
                });
                
                match ureq::post(&url)
                    .set("Authorization", &format!("Bearer {}", openrouter_config.api_key))
                    .send_json(body) 
                {
                    Ok(resp) => {
                        let json: Value = resp.into_json().map_err(|e| e.to_string())?;
                        let response = json["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .trim();
                        match response {
                            "DONE" => Ok(TaskStatus::Done),
                            "STUCK" => Ok(TaskStatus::Stuck),
                            _ => Err(format!("LLM 返回未知状态: {}", response)),
                        }
                    }
                    Err(e) => Err(format!("OpenRouter 调用失败: {}", e)),
                }
            } else {
                Err("OpenRouter 配置未找到".to_string())
            }
        }
        _ => Err("未知的 LLM_BACKEND".to_string()),
    }
}