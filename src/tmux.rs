use std::process::Command;

/// 发送按键命令到指定的tmux窗格
/// 
/// 这个函数会分两步发送：先发送文本，等待一小段时间，然后发送回车键
/// 这样可以解决时序问题，确保消息被正确接收和处理
/// 
/// 参数说明：
/// - text: 要发送的文本内容
/// - pane: 目标tmux窗格ID
pub fn send_keys(text: &str, pane: &str) {
    println!("🔧 发送命令到 tmux pane {}: {}", pane, text);
    
    // 第一步：发送文本内容
    let text_result = Command::new("tmux")
        .args(&["send-keys", "-t", pane, text])
        .output();
    
    match text_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 文本发送成功");
            } else {
                println!("⚠️ 文本发送失败，状态码: {}", output.status);
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ 文本发送错误: {}", stderr.trim());
                }
                return;
            }
        }
        Err(e) => {
            println!("❌ 无法执行文本发送命令: {}", e);
            return;
        }
    }
    
    // 等待一小段时间，确保文本被完全接收
    std::thread::sleep(std::time::Duration::from_millis(150));
    
    // 第二步：发送回车键 (C-m)
    let enter_result = Command::new("tmux")
        .args(&["send-keys", "-t", pane, "C-m"])
        .output();
    
    match enter_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 回车键发送成功");
            } else {
                println!("⚠️ 回车键发送失败，状态码: {}", output.status);
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ 回车键发送错误: {}", stderr.trim());
                }
            }
        }
        Err(e) => {
            println!("❌ 无法执行回车键发送命令: {}", e);
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