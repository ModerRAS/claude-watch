use std::process::Command;

/// 发送按键命令到指定的tmux窗格
/// 
/// 这个函数会发送文本后自动发送回车键
/// 参数说明：
/// - text: 要发送的文本内容
/// - pane: 目标tmux窗格ID
/// 
/// 命令结构：tmux send-keys -t {pane} {text} C-m
/// - C-m 代表 Ctrl+M，即回车键，确保消息被发送
/// 
/// 改进：添加调试信息和错误处理，尝试多种按键组合
pub fn send_keys(text: &str, pane: &str) {
    println!("🔧 发送命令到 tmux pane {}: {}", pane, text);
    
    // 首先尝试发送文本和回车
    let result = Command::new("tmux")
        .args(&["send-keys", "-t", pane, text, "C-m"])
        .output();
    
    match result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 命令发送成功");
                
                // 如果是发送 "Retry"，可能需要额外的按键来确保执行
                if text == "Retry" {
                    // 等待一小段时间，然后尝试不同的按键组合
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    
                    // 尝试 C-j (Ctrl+J)，这也是一个回车键的替代
                    let extra_result = Command::new("tmux")
                        .args(&["send-keys", "-t", pane, "C-j"])
                        .output();
                    
                    match extra_result {
                        Ok(extra_output) => {
                            if extra_output.status.success() {
                                println!("✅ 额外 C-j 发送成功");
                            } else {
                                println!("⚠️ 额外 C-j 发送失败");
                            }
                        }
                        Err(e) => {
                            println!("❌ 发送额外 C-j 失败: {}", e);
                        }
                    }
                    
                    // 再等待一下，尝试 C-d (Ctrl+D)，EOF 信号
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    
                    let eof_result = Command::new("tmux")
                        .args(&["send-keys", "-t", pane, "C-d"])
                        .output();
                    
                    match eof_result {
                        Ok(eof_output) => {
                            if eof_output.status.success() {
                                println!("✅ C-d (EOF) 发送成功");
                            } else {
                                println!("⚠️ C-d (EOF) 发送失败");
                            }
                        }
                        Err(e) => {
                            println!("❌ 发送 C-d (EOF) 失败: {}", e);
                        }
                    }
                }
                
                // 如果有 stderr 输出，也显示一下
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.trim().is_empty() {
                        println!("📝 命令输出: {}", stderr.trim());
                    }
                }
            } else {
                println!("⚠️ 命令发送失败，状态码: {}", output.status);
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ 错误信息: {}", stderr.trim());
                }
            }
        }
        Err(e) => {
            println!("❌ 无法执行 tmux 命令: {}", e);
        }
    }
}

/// 从指定的tmux窗格捕获内容
/// 
/// 这个函数会捕获tmux窗格中的所有文本内容
/// 参数说明：
/// - pane: 目标tmux窗格ID
/// 
/// 命令结构：tmux capture-pane -p -t {pane}
/// - -p: 以纯文本格式输出
/// - -t {pane}: 指定目标窗格
pub fn capture(pane: &str) -> String {
    let out = Command::new("tmux")
        .args(&["capture-pane", "-p", "-t", pane])
        .output()
        .expect("tmux capture failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}