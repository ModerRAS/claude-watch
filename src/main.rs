use dotenvy::dotenv;
use serde_json::{json, Value};
use std::{
    io,
    process::Command,
    thread,
    time::{Duration, Instant},
};
use clap::Parser;

mod config;
use config::Config;


fn send_keys(text: &str, pane: &str) {
    let _ = Command::new("tmux")
        .args(&["send-keys", "-t", pane, text, "C-m"])
        .status();
}

fn capture(pane: &str) -> String {
    let out = Command::new("tmux")
        .args(&["capture-pane", "-p", "-t", pane])
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
/// 简化实现：使用 ollama-rs 库提供的高级 API 接口，支持自定义服务器地址和模型选项
/// 这是一个简化实现，替换了手动 HTTP 请求处理并支持配置
async fn ask_ollama_with_ollama_rs(prompt_text: &str, model: &str, url: &str) -> Result<String, String> {
    // 解析 URL 获取主机和端口
    // 简化实现：解析 URL 为独立的主机和端口参数，符合 ollama-rs API 要求
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

/// 原本实现：直接传递完整 URL 字符串给 Ollama::new
/// 简化实现：解析 URL 为独立的主机和端口参数，符合 ollama-rs API 要求
/// 这是一个简化实现，移除了复杂的 URL 解析逻辑
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

/// 使用 openai-api-rs 库调用 OpenAI 兼容的 API
async fn ask_openai(prompt_text: &str, config: &config::OpenAiConfig) -> Result<String, String> {
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

/// 原本实现：复杂的混合状态判断
/// 简化实现：直接使用 LLM 判断最终状态，集成 ollama-rs
/// 这是一个简化实现，替换了手动 HTTP 请求处理
fn ask_llm_final_status(text: &str, backend: &str, config: &Config) -> Result<TaskStatus, String> {
    
    if backend == "none" {
        // 如果禁用 LLM，使用简单的启发式判断
        return Ok(simple_heuristic_check(text));
    }
    
    let prompt = include_str!("../prompt_optimized.md");
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

/// 命令行参数配置
/// 简化实现：使用clap解析命令行参数，替代环境变量
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    /// tmux pane ID (例如 %0 或 mysess:1.0)
    #[arg(short, long)]
    pane: Option<String>,

    /// LLM 后端选择 [ollama, openai, openrouter, none]
    #[arg(short, long)]
    backend: Option<String>,

    /// 检查间隔(秒) [默认: 5]
    #[arg(short, long)]
    interval: Option<u64>,

    /// 无变化多久算卡住(秒) [默认: 60]
    #[arg(short, long)]
    stuck_sec: Option<u64>,

    /// 最大重试次数 [默认: 10]
    #[arg(short, long)]
    max_retry: Option<usize>,
}

fn main() -> io::Result<()> {
    dotenv().ok();
    let args = Args::parse();

    // 加载配置
    let mut config = Config::load(&args.config).unwrap_or_else(|_| {
        eprintln!("无法加载配置文件 {}，使用默认配置", args.config);
        Config::default()
    });

    // 使用命令行参数覆盖配置（如果提供）
    if let Some(pane) = &args.pane {
        config.tmux.pane = pane.clone();
    }
    if let Some(backend) = &args.backend {
        config.llm.backend = backend.clone();
    }
    if let Some(interval) = args.interval {
        config.monitoring.interval = interval;
    }
    if let Some(stuck_sec) = args.stuck_sec {
        config.monitoring.stuck_sec = stuck_sec;
    }
    if let Some(max_retry) = args.max_retry {
        config.monitoring.max_retry = max_retry;
    }

    let mut last_active = Instant::now();
    let mut retry_count = 0usize;

    println!("开始监控 Claude Code 在 tmux pane {} 中的状态", config.tmux.pane);
    println!("使用 LLM 后端: {}", config.llm.backend);

    // 主监控循环
    run_monitoring_loop(&config, &mut last_active, &mut retry_count)
}

fn run_monitoring_loop(
    config: &Config,
    last_active: &mut Instant,
    retry_count: &mut usize,
) -> io::Result<()> {
    loop {
        let text = capture(&config.tmux.pane);
        
        // 检查 Claude Code 是否仍在活动
        if is_claude_active(&text) {
            // Claude Code 仍在活动
            *last_active = Instant::now();
            *retry_count = 0;
            println!("🔄 Claude Code 正在工作中...");
        } else {
            // Claude Code 不活动，检查是否超时
            if last_active.elapsed() >= Duration::from_secs(config.monitoring.stuck_sec) {
                println!("⏸️ Claude Code 停止工作超过 {} 秒，调用 LLM 判断状态...", config.monitoring.stuck_sec);
                
                match ask_llm_final_status(&text, &config.llm.backend, config) {
                    Ok(TaskStatus::Done) => {
                        println!("✅ LLM 确认任务已完成，进入完成状态监控...");
                        // 进入完成状态监控循环
                        if monitor_completion_state(&config.tmux.pane).is_err() {
                            println!("⚠️ 完成状态监控中断，重新开始正常监控");
                        }
                    }
                    Ok(TaskStatus::Stuck) => {
                        println!("⚠️ LLM 确认任务卡住");
                        if *retry_count < config.monitoring.max_retry {
                            println!("重试 {}/{}", *retry_count + 1, config.monitoring.max_retry);
                            send_keys("Retry", &config.tmux.pane);
                            *retry_count += 1;
                        } else {
                            println!("达到最大重试次数，发送 /compact");
                            send_keys("/compact", &config.tmux.pane);
                            *retry_count = 0;
                        }
                        // 重置状态，重新开始监控
                        *last_active = Instant::now();
                    }
                    Err(e) => {
                        eprintln!("⚠️ 状态判断失败: {}，等待下次检查", e);
                        // 等待更长时间再重试
                        thread::sleep(Duration::from_secs(config.monitoring.stuck_sec));
                    }
                }
            } else {
                let wait_time = config.monitoring.stuck_sec - last_active.elapsed().as_secs();
                println!("⏳ 等待 {} 秒后判断 Claude Code 状态...", wait_time);
            }
        }
        
        thread::sleep(Duration::from_secs(config.monitoring.interval));
    }
}

/// 原本实现：在 LLM 判断为 DONE 后立即退出程序
/// 简化实现：持续监控完成状态，检测画面变化以决定是否重启监控
/// 这是一个简化实现，将程序变为守护进程模式
fn monitor_completion_state(pane: &str) -> Result<(), String> {
    let mut last_hash = 0u64;
    let mut check_count = 0usize;
    
    println!("🔄 进入完成状态监控模式...");
    
    loop {
        let text = capture(pane);
        let hash = seahash::hash(text.as_bytes());
        
        if hash != last_hash {
            // 画面发生变化，说明 Claude Code 可能开始了新任务
            println!("🔍 检测到画面变化，Claude Code 可能开始新任务");
            return Ok(());
        }
        
        last_hash = hash;
        check_count += 1;
        
        // 每检查 10 次报告一次状态
        if check_count % 10 == 0 {
            println!("💤 仍在完成状态，持续监控中... (检查次数: {})", check_count);
        }
        
        // 睡眠 60 秒（1 分钟）
        thread::sleep(Duration::from_secs(60));
    }
}
