use crate::config::{Config, OpenAiConfig, OpenRouterConfig};
use serde_json::{json, Value};
// tokio imported implicitly through async functions

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

/// 手搓 HTTP 请求调用 OpenAI 兼容的 API
async fn ask_openai(system_prompt: &str, user_content: &str, config: &crate::config::OpenAiConfig) -> Result<String, String> {
    use serde_json::{json, Value};
    
    // 检查 API key 是否为空
    if config.api_key.is_empty() {
        return Err("OpenAI API key 未设置".to_string());
    }
    
    // 创建请求体
    let request_body = json!({
        "model": config.model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_content
            }
        ],
        "max_tokens": 4,
        "temperature": 0.0
    });
    
    // 构建完整的 URL
    let url = if config.api_base.ends_with('/') {
        format!("{}chat/completions", config.api_base)
    } else {
        format!("{}/chat/completions", config.api_base)
    };
    
    // 发送 HTTP 请求
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "无法获取错误信息".to_string());
        return Err(format!("API 请求失败，状态码: {}, 错误: {}", status, error_text));
    }
    
    // 解析响应 JSON
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    
    let json_response: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析 JSON 失败: {}, 响应: {}", e, response_text))?;
    
    // 提取结果
    if let Some(choices) = json_response.get("choices").and_then(|v| v.as_array()) {
        if let Some(first_choice) = choices.first() {
            if let Some(message) = first_choice.get("message").and_then(|v| v.as_object()) {
                // 只从 content 字段判断，忽略推理过程
                if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                    if !content.is_empty() {
                        return Ok(content.to_string());
                    }
                }
                
                // 如果 content 为空，忽略推理过程，直接返回 STUCK
                // 因为画面已经停止变化，默认认为卡住
                return Ok("STUCK".to_string());
            }
        }
    }
    
    Err("无法解析 API 响应".to_string())
}

/// 简化的启发式检查（仅在 LLM 不可用时使用）
pub fn simple_heuristic_check(text: &str) -> TaskStatus {
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
        "工作已完成",
        "所有步骤已完成",
        "代码生成完毕",
        "所有文件已创建完成",
        // Claude Code特有的完成模式
        "✅ Task completed",
        "All done",
        "Task completed successfully",
        "Operation completed",
        "Processing complete",
        "Generation complete",
        "Build complete",
        "Compilation complete",
        "Analysis complete",
        "All operations completed",
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
        "无法继续",
        "中断",
    ];
    
    if error_patterns.iter().any(|&pattern| text.contains(pattern)) {
        return TaskStatus::Stuck;
    }
    
    // 检查可能仍在处理中的状态（避免误判为卡住）
    let processing_patterns = [
        "Cogitating",
        "Thinking",
        "Processing",
        "Working",
        "Analyzing",
        "Generating",
        "Compiling",
        "Building",
        "Installing",
        "Tool use",
        "Calling tool",
        "Function call",
        "API call",
        "Reading file",
        "Writing file",
        "Creating file",
        "Editing file",
        "Downloading",
        "Uploading",
        "Checking",
        "Testing",
        "Retry",
        "Escaping",
        "Interrupting",
        "...",
        "▪▪▪",
        "◦◦◦",
    ];
    
    // 如果检测到处理中状态，不轻易判断为卡住
    // 这里返回一个特殊状态，让监控逻辑继续等待
    if processing_patterns.iter().any(|&pattern| text.contains(pattern)) {
        // 简化起见，我们仍然返回 Stuck，但在监控逻辑中需要处理这种情况
        // 更好的做法是增加一个 Processing 状态，但现在先这样处理
        return TaskStatus::Stuck;
    }
    
    // 检查是否有未完成的命令或程序输出
    let lines: Vec<&str> = text.lines().collect();
    let last_few_lines: Vec<&str> = lines.iter().rev().take(5).cloned().collect();
    let last_content = last_few_lines.join("\n");
    
    // 这个逻辑移到了monitor.rs中的check_if_should_skip_llm_call函数中
    // 这里只处理启发式检查，不跳过LLM调用判断
    
    // 如果最后几行看起来像是程序输出的一部分，可能是正常的处理状态
    if last_content.contains('$') || last_content.contains('>') || last_content.contains('#') {
        // 如果有命令提示符，可能是在等待输入，不算卡住
        return TaskStatus::Stuck;
    }
    
    // 如果文本看起来不完整或者有未闭合的结构，可能是执行中
    if text.ends_with("...") || text.ends_with("•") || text.ends_with("▪") {
        // 仍在处理中，不立即判断为卡住
        return TaskStatus::Stuck;
    }
    
    // 默认认为卡住（因为画面已经停止变化了）
    // 但这个逻辑现在更加谨慎，只有在确认没有其他状态时才判断为卡住
    TaskStatus::Stuck
}

/// 使用 LLM 生成激活消息
/// 
/// 这是智能激活功能，让LLM生成一句话来激活卡住的Claude Code
pub async fn ask_llm_for_activation(prompt: &str, backend: &str, config: &Config) -> Result<String, String> {
    match backend {
        "openai" => {
            if let Some(openai_config) = &config.llm.openai {
                ask_openai_for_activation(prompt, openai_config).await
            } else {
                Err("OpenAI配置未找到".to_string())
            }
        },
        "openrouter" => {
            if let Some(openrouter_config) = &config.llm.openrouter {
                ask_openrouter_for_activation(prompt, openrouter_config).await
            } else {
                Err("OpenRouter配置未找到".to_string())
            }
        },
        "ollama" => {
            if let Some(ollama_config) = &config.llm.ollama {
                ask_ollama_with_ollama_rs(prompt, &ollama_config.model, &ollama_config.url).await
            } else {
                Err("Ollama配置未找到".to_string())
            }
        },
        _ => {
            Err(format!("不支持的LLM后端: {}", backend))
        }
    }
}

/// 使用OpenAI生成激活消息
async fn ask_openai_for_activation(prompt: &str, config: &OpenAiConfig) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    // 构建正确的URL - 添加 chat/completions 路径
    let url = if config.api_base.ends_with('/') {
        format!("{}chat/completions", config.api_base)
    } else {
        format!("{}/chat/completions", config.api_base)
    };
    
    let request_body = serde_json::json!({
        "model": &config.model,
        "messages": [
            {
                "role": "system",
                "content": "你是一个Claude Code激活助手。当Claude Code卡住时，你需要生成一句简短、有效的话来激活它。"
            },
            {
                "role": "user", 
                "content": prompt
            }
        ],
        "max_tokens": 50,
        "temperature": 0.1
    });
    
    // 静默构建请求，调试信息已确认功能正常
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", &config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if let Ok(text) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                        return Ok(content.trim().to_string());
                    } else {
                        return Err("OpenAI响应中缺少content字段".to_string());
                    }
                } else {
                    return Err("OpenAI响应解析失败".to_string());
                }
            } else {
                return Err("OpenAI响应读取失败".to_string());
            }
        },
        Err(e) => {
            Err(format!("OpenAI请求失败: {}", e))
        }
    }
}

/// 使用OpenRouter生成激活消息
async fn ask_openrouter_for_activation(prompt: &str, config: &OpenRouterConfig) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let request_body = serde_json::json!({
        "model": &config.model,
        "messages": [
            {
                "role": "system",
                "content": "你是一个Claude Code激活助手。当Claude Code卡住时，你需要生成一句简短、有效的话来激活它。"
            },
            {
                "role": "user", 
                "content": prompt
            }
        ],
        "max_tokens": 50,
        "temperature": 0.1
    });
    
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", &config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if let Ok(text) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                        return Ok(content.trim().to_string());
                    }
                }
            }
            Err("OpenRouter响应解析失败".to_string())
        },
        Err(e) => {
            Err(format!("OpenRouter请求失败: {}", e))
        }
    }
}

/// 使用 LLM 判断 Claude Code 最终状态
/// 
/// 这是最关键的状态判断函数，仅在画面长时间无变化时调用
/// 根据配置的 LLM 后端类型进行判断：
/// - "ollama": 使用 Ollama 服务
/// - "openai": 使用 OpenAI 或兼容服务
/// - "openrouter": 使用 OpenRouter 服务
/// - "none": 使用简单的启发式判断
pub async fn ask_llm_final_status(text: &str, backend: &str, config: &Config) -> Result<TaskStatus, String> {
    if backend == "none" {
        // 如果禁用 LLM，使用简单的启发式判断
        return Ok(simple_heuristic_check(text));
    }
    
    // 只读取一次 system prompt
    let system_prompt = include_str!("../prompt_final.md");

    match backend.as_ref() {
        "ollama" => {
            let model = config.llm.ollama.as_ref().map(|o| o.model.clone()).unwrap_or("qwen2.5:3b".to_string());
            let url = config.llm.ollama.as_ref().map(|o| o.url.clone()).unwrap_or("http://localhost:11434".to_string());
            
            // 对于 Ollama，我们需要将 system 和 user 内容以合适的格式传递
            let ollama_prompt = format!("### 系统指令\n{}\n\n### 用户内容\n{}", system_prompt, text);
            
            match ask_ollama_with_ollama_rs(&ollama_prompt, &model, &url).await {
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
            if let Some(openai_config) = &config.llm.openai {
                match ask_openai(&system_prompt, &text, &openai_config).await {
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
                        {"role": "system", "content": system_prompt},
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
                            .unwrap_or("[空响应]")
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