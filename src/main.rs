use dotenvy::dotenv;
use serde_json::{json, Value};
use std::{
    env,
    io,
    process::Command,
    thread,
    time::{Duration, Instant},
};

macro_rules! var {
    ($k:expr) => {
        env::var($k).unwrap_or_else(|_| panic!("{} not set", $k))
    };
}

fn send_keys(text: &str) {
    let _ = Command::new("tmux")
        .args(&["send-keys", "-t", &var!("PANE"), text, "C-m"])
        .status();
}

fn capture() -> String {
    let out = Command::new("tmux")
        .args(&["capture-pane", "-p", "-t", &var!("PANE")])
        .output()
        .expect("tmux capture failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 原本实现：简单的字符串比较检测画面变化
/// 简化实现：检测 Claude Code 特定的活动模式
/// 这是一个简化实现，专注于 Claude Code 的特定输出格式
fn is_claude_active(text: &str) -> bool {
    // 检测 Claude Code 的特定活动模式：
    // 1. 包含类似 "104s" 的时间格式（数字+s）
    // 2. 包含 tokens 计数
    // 3. 包含 "Processing" 或其他处理状态
    
    let lines: Vec<&str> = text.lines().collect();
    for line in lines.iter().rev().take(10) {
        // 检查是否有类似 "104s" 的格式
        if line.contains('s') && line.chars().any(|c| c.is_ascii_digit()) {
            // 检查是否在同一行有 tokens 计数或其他活动指示
            if line.contains("tokens") || line.contains("Processing") || line.contains("↓") {
                return true;
            }
        }
    }
    
    false
}

/// 原本实现：使用 ureq 手动构建 HTTP 请求发送到 Ollama API
/// 简化实现：使用 ollama-rs 库提供的高级 API 接口
/// 这是一个简化实现，替换了手动 HTTP 请求处理
async fn ask_ollama_with_ollama_rs(prompt_text: &str, model: &str) -> Result<String, String> {
    // 初始化 Ollama 客户端（使用默认配置连接到本地服务）
    let ollama = ollama_rs::Ollama::default();
    
    // 构建生成请求
    let request = ollama_rs::generation::completion::request::GenerationRequest::new(
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

/// 原本实现：复杂的混合状态判断
/// 简化实现：直接使用 LLM 判断最终状态，集成 ollama-rs
/// 这是一个简化实现，替换了手动 HTTP 请求处理
fn ask_llm_final_status(text: &str) -> Result<TaskStatus, String> {
    let backend = var!("LLM_BACKEND");
    
    if backend == "none" {
        // 如果禁用 LLM，使用简单的启发式判断
        return Ok(simple_heuristic_check(text));
    }
    
    let prompt = include_str!("../prompt.md");
    let full_prompt = format!("{}\n\n{}", prompt, text);

    match backend.as_str() {
        "ollama" => {
            // 使用 tokio 运行时来执行异步函数
            let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建运行时失败: {}", e))?;
            let model = "qwen3:7b-instruct-q4_K_M";
            
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
        }
        "openrouter" => {
            let url = "https://openrouter.ai/api/v1/chat/completions";
            let body = json!({
                "model": var!("OPENROUTER_MODEL"),
                "messages": [
                    {"role": "system", "content": prompt},
                    {"role": "user", "content": text}
                ],
                "max_tokens": 4,
                "temperature": 0.0
            });
            
            match ureq::post(&url)
                .set("Authorization", &format!("Bearer {}", var!("OPENROUTER_KEY")))
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
        }
        _ => Err("未知的 LLM_BACKEND".to_string()),
    }
}

#[derive(Debug, PartialEq)]
enum TaskStatus {
    Done,
    Stuck,
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

fn main() -> io::Result<()> {
    dotenv().ok();
    let interval: u64 = var!("INTERVAL").parse().unwrap();
    let stuck_sec: u64 = var!("STUCK_SEC").parse().unwrap();
    let max_retry: usize = var!("MAX_RETRY").parse().unwrap();

    let mut last_active = Instant::now();
    let mut retry_count = 0usize;

    println!("开始监控 Claude Code 在 tmux pane {} 中的状态", var!("PANE"));
    println!("使用 LLM 后端: {}", var!("LLM_BACKEND"));

    loop {
        let text = capture();
        
        // 检查 Claude Code 是否仍在活动
        if is_claude_active(&text) {
            // Claude Code 仍在活动
            last_active = Instant::now();
            retry_count = 0;
            println!("🔄 Claude Code 正在工作中...");
        } else {
            // Claude Code 不活动，检查是否超时
            if last_active.elapsed() >= Duration::from_secs(stuck_sec) {
                println!("⏸️ Claude Code 停止工作超过 {} 秒，调用 LLM 判断状态...", stuck_sec);
                
                match ask_llm_final_status(&text) {
                    Ok(TaskStatus::Done) => {
                        println!("✅ LLM 确认任务已完成，退出监控");
                        break;
                    }
                    Ok(TaskStatus::Stuck) => {
                        println!("⚠️ LLM 确认任务卡住");
                        if retry_count < max_retry {
                            println!("重试 {}/{}", retry_count + 1, max_retry);
                            send_keys("Retry");
                            retry_count += 1;
                        } else {
                            println!("达到最大重试次数，发送 /compact");
                            send_keys("/compact");
                            retry_count = 0;
                        }
                        // 重置状态，重新开始监控
                        last_active = Instant::now();
                    }
                    Err(e) => {
                        eprintln!("⚠️ 状态判断失败: {}，等待下次检查", e);
                        // 等待更长时间再重试
                        thread::sleep(Duration::from_secs(stuck_sec));
                    }
                }
            } else {
                let wait_time = stuck_sec - last_active.elapsed().as_secs();
                println!("⏳ 等待 {} 秒后判断 Claude Code 状态...", wait_time);
            }
        }
        
        thread::sleep(Duration::from_secs(interval));
    }
    
    Ok(())
}
