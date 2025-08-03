use dotenvy::dotenv;
use regex::Regex;
use serde_json::{json, Value};
use std::{
    env,
    io,
    process::Command,
    thread,
    time::Duration,
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

/// 原本实现：复杂的启发式规则和多层判断
/// 简化实现：基于读秒检测的核心逻辑
/// 这是一个核心改进，通过读秒检测大幅减少 LLM 调用
fn has_timer_running(text: &str) -> bool {
    // 检测各种计时器格式
    let timer_patterns = [
        r"⏱\s*\d{1,2}:\d{2}",      // ⏱ 00:42
        r"⌛\s*\d{1,2}:\d{2}",      // ⌛ 00:42
        r"计时[：:]\s*\d+秒",        // 计时: 42秒
        r"时间[：:]\s*\d+秒",        // 时间: 42秒
        r"进度[：:]\s*\d+/\d+",      // 进度: 42/60
        r"\[\d+%\]",                // [42%]
        r"(\d+)%\s*完成",           // 42% 完成
        r"正在处理",                 // 处理中
        r"处理中",                  // 处理中
        r"Working on",              // 英文处理中
        r"In progress",             // 进行中
    ];
    
    let re = Regex::new(&timer_patterns.join("|")).unwrap();
    re.is_match(text)
}

/// 原本实现：复杂的混合状态判断
/// 简化实现：直接使用 LLM 判断最终状态
/// 这是一个简化实现，移除了不必要的中间层
fn ask_llm_final_status(text: &str) -> Result<TaskStatus, String> {
    let backend = var!("LLM_BACKEND");
    
    if backend == "none" {
        // 如果禁用 LLM，使用简单的启发式判断
        return Ok(simple_heuristic_check(text));
    }
    
    let prompt = include_str!("../prompt.md");

    match backend.as_str() {
        "ollama" => {
            let url = var!("OLLAMA_URL");
            let body = json!({
                "model": "qwen3:7b-instruct-q4_K_M",
                "prompt": format!("{}\n\n{}", prompt, text),
                "stream": false,
                "max_tokens": 4,
                "temperature": 0.0
            });
            
            match ureq::post(&url).send_json(body) {
                Ok(resp) => {
                    let json: Value = resp.into_json().map_err(|e| e.to_string())?;
                    let response = json["response"].as_str().unwrap_or("").trim();
                    match response {
                        "DONE" => Ok(TaskStatus::Done),
                        "STUCK" => Ok(TaskStatus::Stuck),
                        _ => Err(format!("LLM 返回未知状态: {}", response)),
                    }
                }
                Err(e) => Err(format!("Ollama 调用失败: {}", e)),
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
    
    // 默认认为卡住（因为读秒已经停止了）
    TaskStatus::Stuck
}

fn main() -> io::Result<()> {
    dotenv().ok();
    let interval: u64 = var!("INTERVAL").parse().unwrap();
    let max_retry: usize = var!("MAX_RETRY").parse().unwrap();

    let mut retry_count = 0usize;
    let mut last_status = String::from("working");

    println!("开始监控 Claude Code 在 tmux pane {} 中的状态", var!("PANE"));
    println!("使用 LLM 后端: {}", var!("LLM_BACKEND"));

    loop {
        let text = capture();
        let has_timer = has_timer_running(&text);

        if has_timer {
            // 有读秒，说明 Claude Code 还在工作
            if last_status != "working" {
                println!("🔄 检测到读秒恢复，Claude Code 继续工作");
                last_status = "working".to_string();
                retry_count = 0;
            }
            println!("⏱️ 读秒运行中，Claude Code 正在工作...");
        } else {
            // 没有读秒，立即调用 LLM 判断状态
            println!("⏸️ 读秒停止，立即调用 LLM 判断状态...");
            
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
                    last_status = "retry_sent".to_string();
                }
                Err(e) => {
                    eprintln!("⚠️ 状态判断失败: {}，等待下次检查", e);
                    last_status = "error".to_string();
                }
            }
        }
        
        thread::sleep(Duration::from_secs(interval));
    }
    
    Ok(())
}
