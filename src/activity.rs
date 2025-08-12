/// 检测 Claude Code 特定的活动模式
/// 
/// 这是核心活动检测器，专注于 Claude Code 的特定输出格式
/// 检测以下特征：
/// 1. 类似 "104s" 的时间格式（数字+s）
/// 2. tokens 计数信息
/// 3. "Processing" 或其他处理状态
/// 4. 传输指示器（如 ↓）
pub fn is_claude_active(text: &str) -> bool {
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