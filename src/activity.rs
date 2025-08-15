/// 检测 Claude Code 特定的活动模式
/// 
/// 这是核心活动检测器，专注于 Claude Code 的特定输出格式
/// 检测以下特征：
/// 1. 类似 "104s" 的时间格式（数字+s）
/// 2. tokens 计数信息
/// 3. "Processing" 或其他处理状态
/// 4. 传输指示器（如 ↓）
/// 5. 思考状态（如 "Cogitating..."、"Thinking..."）
/// 6. 工具调用状态（如 "Tool use"）
/// 7. 重试机制（如 "Retry"）
/// 8. 紧凑模式（如 "/compact"）
/// 9. 编译或构建状态
/// 10. 文件操作状态
pub fn is_claude_active(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    
    // 首先检查整个文本中是否有Claude Code的读秒状态 - 这是最重要的活动指示
    for line in lines.iter() {
        let trimmed = line.trim();
        
        // 检查Claude Code的标准读秒格式：*(状态)… (时间 · 数量 tokens · esc to interrupt)
        // 例如：* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)
        // 或：✶ Perusing… (28s · ⚒ 414 tokens · esc to interrupt)
        if trimmed.contains('(') && trimmed.contains(')') && trimmed.contains("tokens") {
            // 特殊处理：如果是Done状态，不认为是活动状态
            if trimmed.contains("Done") {
                return false; // Done状态不是活动状态
            }
            
            // 检查是否有时间格式（数字+s）在括号内
            let time_pattern = regex::Regex::new(r"\b\d+s\b").unwrap();
            if time_pattern.is_match(trimmed) {
                // 这是标准的Claude Code读秒状态，肯定是活动状态
                return true;
            }
        }
        
        // 兼容旧的简单检测：检查是否有类似 "104s" 的时间格式
        if trimmed.contains('s') && trimmed.chars().any(|c| c.is_ascii_digit()) {
            // 如果有时间格式，进一步检查是否是真正的活动状态
            // 避免误判其他包含数字+s的文本
            if trimmed.contains("tokens") || trimmed.contains("↓") || trimmed.contains("↑") {
                return true;
            }
        }
    }
    
    // 如果没有读秒，只检查最明显的活动状态（只检查最后15行）
    for line in lines.iter().rev().take(15) {
        let trimmed = line.trim();
        
        // 只检查最明确的工具调用状态 - 这些是明确的活动指示
        let clear_activity_patterns = [
            "Tool use",
            "Calling tool",
            "Function call",
            "Reading file",
            "Writing file",
            "Creating file",
            "Editing file",
            "Retry",
            "/compact",
        ];
        
        for pattern in &clear_activity_patterns {
            if trimmed.contains(pattern) {
                return true;
            }
        }
        
        // 检查明显的进度指示器
        let progress_indicators = [
            "▪▪▪",
            "◦◦◦",
            ">>>",
            "***",
        ];
        
        for pattern in &progress_indicators {
            if trimmed.contains(pattern) {
                return true;
            }
        }
    }
    
    false
}