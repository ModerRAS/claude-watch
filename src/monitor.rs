use crate::config::Config;
use crate::activity::is_claude_active;
use crate::llm::ask_llm_final_status;
use crate::llm::TaskStatus;
use crate::tmux::{capture, send_keys};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};
use std::io;

/// 全局状态，用于追踪时间变化
static mut TIME_TRACKER: Option<HashMap<String, u64>> = None;

/// 提取Claude Code执行条中的时间值
fn extract_execution_time(text: &str) -> Option<u64> {
    // 匹配格式：(数字s)
    let time_pattern = regex::Regex::new(r"\((\d+)s\)").unwrap();
    if let Some(caps) = time_pattern.captures(text) {
        if let Some(time_str) = caps.get(1) {
            return time_str.as_str().parse::<u64>().ok();
        }
    }
    None
}

/// 检查时间是否在递增（表明Claude Code在工作）
fn is_time_increasing(current_text: &str, pane: &str) -> bool {
    unsafe {
        if TIME_TRACKER.is_none() {
            TIME_TRACKER = Some(HashMap::new());
        }
        
        if let Some(ref mut tracker) = TIME_TRACKER {
            let current_time = extract_execution_time(current_text);
            
            if let Some(current) = current_time {
                let key = pane.to_string();
                
                if let Some(&previous_time) = tracker.get(&key) {
                    // 如果时间比上次大，说明在递增
                    if current > previous_time {
                        tracker.insert(key, current);
                        return true;
                    }
                } else {
                    // 第一次记录时间
                    tracker.insert(key, current);
                    return true; // 第一次看到时间，认为是活动的
                }
            }
        }
    }
    
    false
}

/// 运行主监控循环
/// 
/// 这是程序的核心监控逻辑，持续检查 Claude Code 的状态：
/// 1. 定期捕获 tmux 窗格内容
/// 2. 检测 Claude Code 是否活跃
/// 3. 如果无活动超过指定时间，调用 LLM 判断状态
/// 4. 根据判断结果采取相应行动
pub async fn run_monitoring_loop(
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
                
                // 关键改进：检查时间是否在递增，这是最可靠的活动指示
                if is_time_increasing(&text, &config.tmux.pane) {
                    println!("🔄 检测到时间在递增，Claude Code 正在工作中，跳过 LLM 调用...");
                    *last_active = Instant::now();
                    thread::sleep(Duration::from_secs(config.monitoring.interval));
                    continue;
                }
                
                // 如果时间没有递增，再进行其他检查
                let should_skip_llm = check_if_should_skip_llm_call(&text);
                
                if should_skip_llm {
                    println!("🔄 检测到可能仍在处理的状态，跳过 LLM 调用，继续观察...");
                    // 重置计时器，给予更多时间
                    *last_active = Instant::now();
                    thread::sleep(Duration::from_secs(config.monitoring.interval));
                    continue;
                }
                
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
                            // 首先尝试智能激活：让LLM直接对终端说话
                            println!("尝试智能激活：让LLM直接对终端说话...");
                            match attempt_llm_activation(config, &config.tmux.pane).await {
                                Ok(true) => {
                                    println!("✅ LLM智能激活成功，Claude恢复响应");
                                    *last_active = Instant::now();
                                    *retry_count = 0; // 重置重试计数
                                },
                                Ok(false) => {
                                    println!("⚠️ LLM智能激活无效，尝试传统Retry命令");
                                    // 如果智能激活无效，再尝试传统Retry
                                    println!("重试 {}/{}", *retry_count + 1, config.monitoring.max_retry);
                                    send_keys("Retry", &config.tmux.pane);
                                    *retry_count += 1;
                                    
                                    // 发送Retry后，等待一段时间让Claude响应
                                    println!("等待 {} 秒让 Claude 响应 Retry 命令...", config.monitoring.stuck_sec);
                                    thread::sleep(Duration::from_secs(config.monitoring.stuck_sec));
                                    
                                    // 检查Retry是否有效 - 严格判断是否有实质性进展
                                    let response_text = capture(&config.tmux.pane);
                                    if has_substantial_progress(&response_text) {
                                        println!("✅ Retry 命令有效，Claude 有实质性进展");
                                        *last_active = Instant::now();
                                    } else {
                                        println!("⚠️ Retry 命令无效或只有读秒变化，仍然认为卡住");
                                        // 不重置计时器，让系统继续判断，下次会再次进入卡住检测
                                    }
                                },
                                Err(e) => {
                                    println!("⚠️ LLM智能激活失败: {}，尝试传统Retry命令", e);
                                    // 如果LLM激活失败，回退到传统Retry
                                    println!("重试 {}/{}", *retry_count + 1, config.monitoring.max_retry);
                                    send_keys("Retry", &config.tmux.pane);
                                    *retry_count += 1;
                                    
                                    thread::sleep(Duration::from_secs(config.monitoring.stuck_sec));
                                    
                                    let response_text = capture(&config.tmux.pane);
                                    if has_substantial_progress(&response_text) {
                                        println!("✅ Retry 命令有效，Claude 有实质性进展");
                                        *last_active = Instant::now();
                                    } else {
                                        println!("⚠️ Retry 命令无效或只有读秒变化，仍然认为卡住");
                                    }
                                }
                            }
                        } else {
                            // 达到最大重试次数，启用高级恢复策略
                            println!("达到最大重试次数，启用高级恢复策略...");
                            
                            // 尝试高级解决方案
                            let advanced_solutions = vec![
                                ("请继续你的工作", "LLM温柔提醒"),
                                ("你好，看起来你可能卡住了，请继续处理任务", "LLM明确提醒"),
                                ("/compact", "发送 /compact 命令"),
                                ("Escaping", "发送 Escaping 命令"),
                                ("Ctrl+C", "发送 Ctrl+C 中断当前操作"),
                            ];
                            
                            let mut solution_found = false;
                            for (command, description) in advanced_solutions {
                                println!("尝试高级解决方案: {}", description);
                                send_keys(command, &config.tmux.pane);
                                
                                // 等待响应
                                thread::sleep(Duration::from_secs(config.monitoring.stuck_sec));
                                
                                let solution_text = capture(&config.tmux.pane);
                                if has_substantial_progress(&solution_text) {
                                    println!("✅ 高级解决方案有效: {}", description);
                                    *last_active = Instant::now();
                                    solution_found = true;
                                    break;
                                } else {
                                    println!("⚠️ 高级解决方案无效: {}", description);
                                }
                            }
                            
                            if !solution_found {
                                println!("❌ 所有高级解决方案都无效，进入守护模式");
                                println!("💡 建议：可能需要手动干预或重启Claude Code");
                                // 进入守护模式，不重置计时器，避免无限循环
                            }
                            
                            *retry_count = 0; // 重置重试计数，但只在有实质性进展时重置计时器
                        }
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

/// 监控完成状态
/// 
/// 在 LLM 确认任务完成后，进入守护模式监控：
/// 持续检查画面是否有变化，如果有变化说明 Claude Code 可能开始新任务
/// 这是守护进程模式的核心功能
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

/// 检查是否应该跳过 LLM 调用，避免误判为卡住
/// 
/// 这是防止误判的关键函数，检测可能的中间状态：
/// 1. 深度思考状态
/// 2. 长时间处理的工具调用
/// 3. 网络请求或文件操作
/// 4. 编译或构建过程
pub fn check_if_should_skip_llm_call(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let last_lines: Vec<&str> = lines.iter().rev().take(10).cloned().collect();
    let last_content = last_lines.join("\n");
    
    // 使用正则表达式检查Claude Code的标准执行条格式
    // 格式：*(状态)… (时间 · tokens · esc to interrupt)
    let execution_bar_pattern = regex::Regex::new(r"\*[^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)").unwrap();
    if execution_bar_pattern.is_match(&last_content) {
        return true;
    }
    
    // 作为备选，检查更宽松的模式：包含时间和tokens的括号内容
    let time_tokens_pattern = regex::Regex::new(r"\([^)]*\d+s[^)]*tokens[^)]*\)").unwrap();
    if time_tokens_pattern.is_match(&last_content) {
        return true;
    }
    
    // 检查是否有未完成的输出
    if last_content.ends_with("...") || 
       last_content.ends_with("▪") || 
       last_content.ends_with("◦") ||
       last_content.ends_with("•") ||
       last_content.ends_with(">") ||
       last_content.ends_with("$") {
        return true;
    }
    
    // 检查是否有时间计数器（如 "104s"）但没有其他活动指示
    // 这种情况可能是在等待外部操作完成
    let time_pattern = regex::Regex::new(r"\d+s").unwrap();
    if time_pattern.is_match(&last_content) {
        // 如果有时间计数器但没有明显的完成或错误标志，可能仍在处理
        return !last_content.contains("Error") && 
               !last_content.contains("Failed") &&
               !last_content.contains("Done") &&
               !last_content.contains("Completed");
    }
    
    // 检查是否有命令行提示符，可能是在等待用户输入
    if last_content.contains('$') || last_content.contains('>') || last_content.contains('#') {
        return true;
    }
    
    // 如果以上都不匹配，则不跳过 LLM 调用
    false
}

/// 使用LLM智能激活卡住的Claude Code
/// 
/// 这是核心的智能激活功能，当Claude Code卡住时，
/// 直接调用LLM让它对终端说话，从而激活Claude Code
async fn attempt_llm_activation(config: &Config, pane: &str) -> Result<bool, String> {
    println!("🤖 调用LLM生成激活消息...");
    
    // 构建激活prompt
    let activation_prompt = r#"Claude Code在处理任务时似乎卡住了，需要你生成一句简短而有效的话来激活它。

具体场景：
- Claude Code可能在深度思考、执行工具调用或处理复杂任务时暂时停止响应
- 需要一句自然的、友好的提醒来让它重新开始工作
- 应该像是用户在和Claude Code对话，而不是机械的命令

要求：
1. 生成一句简短、自然、友好的话（10-20个字为佳）
2. 语气要温和，像是在和AI助手对话
3. 内容应该是提醒或询问，让Claude意识到需要继续工作
4. 避免使用"卡住"、"错误"、"问题"等负面词汇
5. 不要包含特殊命令符号（如/、\、#等）

示例：
"请继续处理任务"
"你好，请继续工作"
"看起来可以继续了"
"请继续你的工作"

请只返回要说的话，不要任何解释或其他内容。"#;
    
    // 调用LLM生成激活消息
    match crate::llm::ask_llm_for_activation(activation_prompt, &config.llm.backend, config).await {
        Ok(activation_msg) => {
            let activation_message = activation_msg;
            println!("🤖 LLM生成激活消息: {}", activation_message);
            
            // 发送激活消息到终端
            send_keys(&activation_message, pane);
            
            // 等待Claude响应
            println!("⏳ 等待Claude对激活消息的响应...");
            thread::sleep(Duration::from_secs(config.monitoring.stuck_sec));
            
            // 检查激活是否有效
            let response_text = capture(pane);
            if has_substantial_progress(&response_text) {
                println!("✅ LLM激活成功！Claude有实质性进展");
                Ok(true)
            } else {
                println!("⚠️ LLM激活后无实质性进展");
                Ok(false)
            }
        },
        Err(e) => {
            Err(format!("LLM调用失败: {}", e))
        }
    }
}

/// 检查是否有实质性的进展，而不只是时间计数器
/// 
/// 这个函数用来区分真正的活动恢复和虚假的时间计数器变化
/// 核心原则：只有当有新的实质性内容时，才认为是真正的进展
pub fn has_substantial_progress(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let recent_lines: Vec<&str> = lines.iter().rev().take(5).cloned().collect();
    let recent_content = recent_lines.join("\n");
    
    // 检查是否有新的实质性输出（不只是时间计数器）
    let substantial_indicators = [
        // 新的思考状态
        "Cogitating",
        "Thinking",
        "分析中",
        "思考中",
        
        // 新的工具调用
        "Tool use",
        "Calling tool",
        "Function call",
        
        // 新的文件操作
        "Reading file",
        "Writing file",
        "Creating file",
        "Editing file",
        
        // 新的处理状态
        "Compiling",
        "Building",
        "Installing",
        "Generating",
        
        // 新的网络操作
        "Downloading",
        "Uploading",
        "Fetching",
        
        // 新的命令执行
        "$",
        ">",
        "#",
        
        // 明显的进展指示
        "✅",
        "完成",
        "已完成",
        "Finished",
        "Completed",
        
        // 错误信息（也算进展，说明状态改变了）
        "Error:",
        "error:",
        "Failed",
        "failed",
    ];
    
    for indicator in &substantial_indicators {
        if recent_content.contains(indicator) {
            return true;
        }
    }
    
    // 检查是否有新的大段文本输出（不只是时间计数器）
    // 如果最近几行有实质性的内容变化，而不仅仅是时间计数
    for line in recent_lines {
        let trimmed = line.trim();
        if trimmed.len() > 10 && !is_just_time_counter(trimmed) {
            return true;
        }
    }
    
    false
}

/// 检查是否只是时间计数器，没有实质性内容
pub fn is_just_time_counter(text: &str) -> bool {
    let trimmed = text.trim();
    
    // 检查是否主要是时间计数器格式
    let time_pattern = regex::Regex::new(r"^\*?[^a-zA-Z]*(\d+s)[^a-zA-Z]*(.*)$").unwrap();
    if let Some(caps) = time_pattern.captures(trimmed) {
        let _time_part = &caps[1]; // "104s" 部分
        let rest_part = &caps[2]; // 剩余部分
        
        // 如果剩余部分只有很少的实质性内容，认为只是时间计数器
        let substantial_content = rest_part.contains("tokens") || 
                                rest_part.contains("Processing") ||
                                rest_part.contains("↓") ||
                                rest_part.len() > 20; // 如果剩余部分较长，认为有实质性内容
        
        return !substantial_content;
    }
    
    false
}