use claude_watch::{has_substantial_progress, is_just_time_counter, check_if_should_skip_llm_call};

#[test]
fn test_substantial_progress_true_cases() {
    // 测试应该返回true的情况（有实质性进展）
    let test_cases = vec![
        // 思考状态
        ("Cogitating... some content", true),
        ("Thinking about the problem", true),
        ("分析中复杂逻辑", true),
        ("思考中解决方案", true),
        
        // 工具调用
        ("Tool use: Reading file", true),
        ("Calling tool: function_name", true),
        ("Function call: api_request", true),
        ("API call: https://example.com", true),
        ("Reading file: src/main.rs", true),
        ("Writing file: output.txt", true),
        ("Creating file: new_file.md", true),
        ("Editing file: existing.rs", true),
        
        // 处理状态
        ("Compiling project", true),
        ("Building application", true),
        ("Installing dependencies", true),
        ("Downloading package", true),
        ("Uploading data", true),
        ("Generating code", true),
        ("Fetching data", true),
        
        // 命令执行
        ("$ ls -la", true),
        ("> cargo build", true),
        ("# npm install", true),
        
        // 完成指示
        ("✅ Task completed", true),
        ("完成工作", true),
        ("已完成处理", true),
        ("Finished processing", true),
        ("Completed successfully", true),
        
        // 错误状态（也算进展）
        ("Error: compilation failed", true),
        ("error: something went wrong", true),
        ("Failed to execute", true),
        ("failed: operation failed", true),
        
        // 长文本输出（大于10字符且不是纯时间计数器）
        ("This is a long output with substantial content", true),
        ("Processing multiple files and generating reports", true),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(has_substantial_progress(input), expected, 
                   "Failed for input: '{}'", input);
    }
}

#[test]
fn test_substantial_progress_false_cases() {
    // 测试应该返回false的情况（没有实质性进展）
    let test_cases = vec![
        // 空或几乎空的输入
        ("", false),
        ("   ", false),
        ("\n\n\n", false),
        ("short", false),
        
        // 只有简单文本，没有活动指示
        ("Just some text", false),
        ("Hello world", false),
        ("No activity here", false),
        
        // 纯数字（没有上下文）
        ("123", false),
        ("4567", false),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(has_substantial_progress(input), expected, 
                   "Failed for input: '{}'", input);
    }
}

#[test]
fn test_substantial_progress_multi_line() {
    // 测试多行文本的实质性进展检测
    let test_cases = vec![
        // 有实质性进展的多行文本
        (
            "Line 1\nLine 2\nTool use: Reading file\nLine 4",
            true
        ),
        (
            "Some text\nCogitating... \nMore content",
            true
        ),
        (
            "Previous content\n$ ls -la\nFinal line",
            true
        ),
        
        // 没有实质性进展的多行文本
        (
            "Just some text\nMore text\nFinal line\nNo activity",
            false
        ),
        (
            "Line 1\nLine 2\nLine 3\nLine 4",
            false
        ),
        
        // 混合情况（有进展）
        (
            "No activity here\nBut then: Tool use: Reading file\nAnd more",
            true
        ),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(has_substantial_progress(input), expected, 
                   "Failed for multi-line input:\n{}", input);
    }
}

#[test]
fn test_time_counter_detection() {
    // 测试时间计数器检测
    let test_cases = vec![
        // 是纯时间计数器的情况
        ("* 104s", true),
        ("169s", true),
        ("234s", true),
        ("* 56s ·", true),
        ("89s tokens", false), // 有tokens不算纯时间计数器
        ("343s · ↑ 14.2k tokens", false), // 有tokens和箭头不算纯时间计数器
        ("56s Processing", false), // 有处理状态不算纯时间计数器
        ("* 169s · substantial content here", false), // 有实质性内容
        ("Not a time counter", false),
        ("Some text 104s", false), // 不是主要的时间计数器格式
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_just_time_counter(input), expected, 
                   "Failed for time counter detection: '{}'", input);
    }
}

#[test]
fn test_skip_llm_call_detection() {
    // 测试是否应该跳过LLM调用
    let test_cases = vec![
        // 应该跳过LLM调用的情况
        (
            "Cogitating...\nThinking...\nMore content",
            true
        ),
        (
            "Compiling project\nBuilding application",
            true
        ),
        (
            "Tool use: Reading file\nFunction call: api",
            true
        ),
        (
            "Downloading package\nInstalling dependencies",
            true
        ),
        (
            "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)",
            true
        ),
        (
            "Processing queue\nQueue: 5 items",
            true
        ),
        (
            "...\n▪▪▪\n◦◦◦",
            true
        ),
        (
            "Ends with...\nEnds with ▪\n$ command prompt",
            true
        ),
        
        // 不应该跳过LLM调用的情况
        (
            "Just some text\nNo activity indicators",
            false
        ),
        (
            "Error message\nBut no thinking state",
            false
        ),
        (
            "Task completed\nAll done",
            false
        ),
        (
            "Simple output\nNo complex processing",
            false
        ),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(check_if_should_skip_llm_call(input), expected, 
                   "Failed for skip LLM call detection:\n{}", input);
    }
}

#[test]
fn test_real_world_scenarios() {
    // 测试真实世界的场景
    let test_cases = vec![
        // 场景1: Claude Code正在深度思考
        (
            "* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n\
             \n\
             The user is asking me to continue working.",
            true  // 应该跳过LLM调用，因为有读秒
        ),
        
        // 场景2: 工具调用中
        (
            "Tool use: Reading file\n\
             * Cogitating… (234s · ↓ 11.2k tokens · esc to interrupt)",
            true  // 应该跳过LLM调用
        ),
        
        // 场景3: 编译过程中
        (
            "Compiling project...\n\
             Building application...\n\
             * Building… (343s · ↑ 14.2k tokens · esc to interrupt)",
            true  // 应该跳过LLM调用
        ),
        
        // 场景4: 可能卡住了
        (
            "Some previous output\n\
             \n\
             No activity for a while\n\
             Just waiting...",
            false  // 不应该跳过LLM调用
        ),
        
        // 场景5: 有实质性进展
        (
            "Previous content\n\
             Tool use: Creating new file\n\
             File created successfully",
            true  // 有实质性进展
        ),
        
        // 场景6: 只有时间计数器变化
        (
            "* 104s\n\
             * 105s\n\
             * 106s",
            false  // 没有实质性进展
        ),
    ];
    
    for (input, skip_llm, has_progress) in vec![
        ("* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)\n\nThe user is asking me to continue working.", true, true),
        ("Tool use: Reading file\n* Cogitating… (234s · ↓ 11.2k tokens · esc to interrupt)", true, true),
        ("Compiling project...\nBuilding application...\n* Building… (343s · ↑ 14.2k tokens · esc to interrupt)", true, true),
        ("Some previous output\n\nNo activity for a while\nJust waiting...", false, false),
        ("Previous content\nTool use: Creating new file\nFile created successfully", true, true),
        ("* 104s\n* 105s\n* 106s", false, false),
    ] {
        assert_eq!(check_if_should_skip_llm_call(input), skip_llm, 
                   "Failed skip LLM detection for:\n{}", input);
        assert_eq!(has_substantial_progress(input), has_progress, 
                   "Failed progress detection for:\n{}", input);
    }
}

#[test]
fn test_edge_cases() {
    // 测试边界情况
    let test_cases = vec![
        // 空输入
        ("", false, false),
        
        // 只有空格
        ("   ", false, false),
        
        // 只有换行
        ("\n\n\n", false, false),
        
        // 非常短的文本
        ("a", false, false),
        ("hi", false, false),
        
        // 混合情况
        ("No activity\nBut then: Tool use: Reading file", true, true),
        ("Just text\nBut ends with $", true, true),
        ("Cogitating", true, true),
        ("Error: something", true, true),
    ];
    
    for (input, skip_llm, has_progress) in test_cases {
        assert_eq!(check_if_should_skip_llm_call(input), skip_llm, 
                   "Failed edge case for skip LLM: '{}'", input);
        assert_eq!(has_substantial_progress(input), has_progress, 
                   "Failed edge case for progress: '{}'", input);
    }
}