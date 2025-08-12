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
pub fn send_keys(text: &str, pane: &str) {
    let _ = Command::new("tmux")
        .args(&["send-keys", "-t", pane, text, "C-m"])
        .status();
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