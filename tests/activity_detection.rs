use claude_watch::is_claude_active;

#[test]
fn test_activity_detection_standard_format() {
    // 测试标准Claude Code读秒格式
    let test_cases = vec![
        // 标准格式：*(状态)… (时间 · tokens · esc to interrupt)
        ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
        ("* Cogitating… (169s · ↓ 8.7k tokens · esc to interrupt)", true),
        ("* Processing… (56s · ↑ 2.1k tokens · esc to interrupt)", true),
        ("* Compiling… (89s · ↓ 5.3k tokens · esc to interrupt)", true),
        ("* Building… (234s · ↑ 11.8k tokens · esc to interrupt)", true),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for input: {}", input);
    }
}

#[test]
fn test_activity_detection_simple_time_format() {
    // 测试简单时间格式（兼容性检测）
    let test_cases = vec![
        // 简单格式：有tokens或其他标志
        ("104s · ↓ 4.9k tokens", true),
        ("56s · ↑ 2.3k tokens", true),
        ("234s Processing tokens", true),
        ("89s ↓ tokens", true),
        
        // 没有tokens标志的应该返回false
        ("104s some text", false),
        ("56s alone", false),
        ("just some text", false),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for input: {}", input);
    }
}

#[test]
fn test_activity_detection_tool_calls() {
    // 测试工具调用检测
    let test_cases = vec![
        // 工具调用状态
        ("Tool use: Reading file", true),
        ("Calling tool: function_name", true),
        ("Function call: api_request", true),
        ("Reading file: src/main.rs", true),
        ("Writing file: output.txt", true),
        ("Creating file: new_file.md", true),
        ("Editing file: existing.rs", true),
        
        // 重试和命令
        ("Retry", true),
        ("/compact", true),
        ("Escaping", false), // 根据当前逻辑，Escaping不被识别为活动状态
        ("Interrupting", false), // 根据当前逻辑，Interrupting不被识别为活动状态
        
        // 进度指示器
        ("▪▪▪", true),
        ("◦◦◦", true),
        (">>>", true),
        ("***", true),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for input: {}", input);
    }
}

#[test]
fn test_activity_detection_multi_line() {
    // 测试多行文本检测
    let test_cases = vec![
        // 读秒在任何一行都应该被检测到
        (
            "Some previous text\n* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)\nMore text",
            true
        ),
        (
            "Line 1\nLine 2\n169s · ↓ 8.7k tokens\nLine 4",
            true
        ),
        (
            "* Cogitating… (56s · ↑ 2.1k tokens · esc to interrupt)\nSome other content\nMore content",
            true
        ),
        
        // 工具调用在最后几行
        (
            "Some text\nMore text\nTool use: Reading file\nFinal line",
            true
        ),
        
        // 没有活动状态
        (
            "Just some text\nMore text\nFinal line\nNo activity here",
            false
        ),
        (
            "Error message\nSome content\nNo tokens or time",
            false
        ),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for multi-line input:\n{}", input);
    }
}

#[test]
fn test_activity_detection_edge_cases() {
    // 测试边界情况
    let test_cases = vec![
        // 空输入
        ("", false),
        ("   ", false),
        ("\n\n\n", false),
        
        // 只有部分匹配
        ("(343s) but no tokens", true), // 根据当前逻辑，括号中的时间会被识别
        ("tokens but no time", false),
        ("* Herding… but no parentheses", false),
        
        // 无效格式
        ("* Herding… (not-a-number s · tokens)", false),
        ("* Herding… (343s)", false), // 缺少tokens
        ("(343s · tokens)", true), // 根据当前逻辑，包含时间就会被识别
        
        // 混合内容
        ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt) and some extra text", true),
        ("Prefix: * Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for edge case: '{}'", input);
    }
}

#[test]
fn test_activity_detection_queue_states() {
    // 测试Queue状态检测
    let test_cases = vec![
        // Queue相关状态
        ("Queue processing", false), // Queue不在活动检测中，只有读秒才是活动
        ("Queued tasks", false),
        ("Processing queue", false),
        ("queue: 5 items", false),
        
        // 但是如果有tokens标志，应该是活动的
        ("Queue processing (56s · ↑ 2.3k tokens)", true),
        ("Queued: 3 items (89s · ↓ 1.5k tokens)", true),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for queue state: '{}'", input);
    }
}

#[test]
fn test_activity_detection_real_world_scenarios() {
    // 测试真实场景
    let test_cases = vec![
        // 真实的Claude Code输出
        (
            "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n\
             \n\
             The user is asking me to continue working.",
            true
        ),
        (
            "Tool use: Reading file\n\
             * Cogitating… (234s · ↓ 11.2k tokens · esc to interrupt)",
            true
        ),
        (
            "Error: Something went wrong\n\
             No activity detected",
            false
        ),
        (
            "Compiling project...\n\
             * Building… (343s · ↑ 14.2k tokens · esc to interrupt)",
            true
        ),
        (
            "Task completed\n\
             All done",
            false
        ),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "Failed for real-world scenario:\n{}", input);
    }
}