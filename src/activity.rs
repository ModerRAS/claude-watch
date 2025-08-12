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
    for line in lines.iter().rev().take(15) {
        let trimmed = line.trim();
        
        // 检查是否有类似 "104s" 的时间格式
        if trimmed.contains('s') && trimmed.chars().any(|c| c.is_ascii_digit()) {
            // 检查是否在同一行有 tokens 计数或其他活动指示
            if trimmed.contains("tokens") || trimmed.contains("Processing") || trimmed.contains("↓") {
                return true;
            }
        }
        
        // 检查思考和处理状态
        let processing_patterns = [
            "Cogitating",
            "Thinking", 
            "Processing",
            "Working",
            "Analyzing",
            "Generating",
            "Compiling",
            "Building",
            "Installing",
            "Downloading",
            "Uploading",
            "Checking",
            "Testing",
        ];
        
        for pattern in &processing_patterns {
            if trimmed.contains(pattern) {
                return true;
            }
        }
        
        // 检查工具调用和系统状态
        let system_patterns = [
            "Tool use",
            "Calling tool",
            "Function call",
            "API call",
            "HTTP request",
            "File operation",
            "Reading file",
            "Writing file",
            "Creating file",
            "Editing file",
        ];
        
        for pattern in &system_patterns {
            if trimmed.contains(pattern) {
                return true;
            }
        }
        
        // 检查重试和特殊命令
        let command_patterns = [
            "Retry",
            "/compact",
            "Escaping",
            "Interrupting",
        ];
        
        for pattern in &command_patterns {
            if trimmed.contains(pattern) {
                return true;
            }
        }
        
        // 检查进度指示器
        let progress_patterns = [
            ".",
            "..",
            "...",
            "▪",
            "▫",
            "•",
            "◦",
            "▪▪▪",
            "◦◦◦",
        ];
        
        for pattern in &progress_patterns {
            if trimmed.contains(pattern) && trimmed.len() < 50 {
                return true;
            }
        }
        
        // 检查数字计数器（如 tokens 数、文件数等）
        if trimmed.chars().any(|c| c.is_ascii_digit()) && 
           (trimmed.contains("files") || trimmed.contains("items") || 
            trimmed.contains("entries") || trimmed.contains("results")) {
            return true;
        }
        
        // 检查网络活动指示
        if trimmed.contains("↓") || trimmed.contains("↑") || trimmed.contains("↔") {
            return true;
        }
    }
    
    false
}