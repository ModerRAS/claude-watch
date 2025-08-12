use std::process::Command;

/// 发送按键命令到指定的tmux窗格
pub fn send_keys(text: &str, pane: &str) {
    let _ = Command::new("tmux")
        .args(&["send-keys", "-t", pane, text, "C-m"])
        .status();
}

/// 从指定的tmux窗格捕获内容
pub fn capture(pane: &str) -> String {
    let out = Command::new("tmux")
        .args(&["capture-pane", "-p", "-t", pane])
        .output()
        .expect("tmux capture failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}