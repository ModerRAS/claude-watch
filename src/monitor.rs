use crate::config::Config;
use crate::activity::is_claude_active;
use crate::llm::ask_llm_final_status;
use crate::llm::TaskStatus;
use crate::tmux::{capture, send_keys};
use std::thread;
use std::time::{Duration, Instant};
use std::io;

/// 运行主监控循环
/// 
/// 这是程序的核心监控逻辑，持续检查 Claude Code 的状态：
/// 1. 定期捕获 tmux 窗格内容
/// 2. 检测 Claude Code 是否活跃
/// 3. 如果无活动超过指定时间，调用 LLM 判断状态
/// 4. 根据判断结果采取相应行动
pub fn run_monitoring_loop(
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