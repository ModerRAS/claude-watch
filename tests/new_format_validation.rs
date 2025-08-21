//! 新格式适配验证测试
//! 
//! 专门测试Claude Code新状态标识格式的适配实现

use claude_watch::activity::is_claude_active;
use claude_watch::monitor::{
    extract_execution_time, 
    check_if_should_skip_llm_call, 
    has_substantial_progress
};

/// 测试新格式：只有 (esc to interrupt) 的简化格式
#[test]
fn test_new_format_simple_esc_only() {
    let test_cases = vec![
        // 新格式：只有 esc to interrupt，没有 tokens
        ("* Processing… (56s · esc to interrupt)", true),
        ("✶ Thinking… (123s · esc to interrupt)", true),
        ("✻ Cogitating… (89s · esc to interrupt)", true),
        ("✽ Herding… (234s · esc to interrupt)", true),
        ("· Meandering… (45s · esc to interrupt)", true),
        
        // 新格式：没有时间，只有 esc to interrupt
        ("* Processing… (esc to interrupt)", true),
        ("✶ Thinking… (esc to interrupt)", true),
        
        // 边界情况：esc to interrupt 在其他位置
        ("* Processing… (56s · tokens · esc to interrupt)", true), // 标准格式应该仍然支持
        ("Some text (esc to interrupt) more text", false), // 不在执行条格式中
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "新格式测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试向后兼容性：确保旧格式仍然支持
#[test]
fn test_backward_compatibility() {
    let test_cases = vec![
        // 标准旧格式
        ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
        ("✶ Perusing… (28s · ⚒ 414 tokens · esc to interrupt)", true),
        ("✻ Philosophising… (475s · ↓ 7 tokens · esc to interrupt)", true),
        ("✽ Cogitating… (169s · ↑ 8.7k tokens · esc to interrupt)", true),
        
        // 简化的旧格式
        ("* Herding (169s) tokens esc to interrupt", true),
        ("* Cogitating (343s) tokens esc to interrupt", true),
        
        // 只有时间的格式
        ("104s · ↓ 4.9k tokens", true),
        ("56s · ↑ 2.3k tokens", true),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(is_claude_active(input), expected, 
                   "向后兼容性测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试新格式时间提取
#[test]
fn test_new_format_time_extraction() {
    let test_cases = vec![
        // 新格式：只有 esc to interrupt
        ("* Processing… (56s · esc to interrupt)", Some(56)),
        ("✶ Thinking… (123s · esc to interrupt)", Some(123)),
        ("* Processing… (esc to interrupt)", None), // 没有时间信息
        
        // 标准格式应该仍然工作
        ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", Some(343)),
        ("✶ Perusing… (28s · ⚒ 414 tokens · esc to interrupt)", Some(28)),
        
        // 边界情况
        ("No time here", None),
        ("Invalid time (abc s · esc to interrupt)", None),
    ];
    
    for (input, expected) in test_cases {
        let result = extract_execution_time(input);
        assert_eq!(result, expected, 
                   "时间提取测试失败: input='{}', expected={:?}, got={:?}", 
                   input, expected, result);
    }
}

/// 测试新格式跳过LLM调用逻辑
#[test]
fn test_new_format_skip_llm_logic() {
    let test_cases = vec![
        // 新格式：应该跳过LLM调用
        ("* Processing… (56s · esc to interrupt)", true),
        ("✶ Thinking… (123s · esc to interrupt)", true),
        
        // 标准格式：应该跳过LLM调用
        ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
        
        // 中断状态：不应该跳过LLM调用（即使有执行条格式）
        ("* Processing… (56s · esc to interrupt)\nInterrupted by user", false),
        ("* Processing… (56s · esc to interrupt)\nAborted by user", false),
        ("* Processing… (56s · esc to interrupt)\nCancelled by user", false),
        ("Interrupted by user", false),
        
        // 只有命令提示符：应该跳过LLM调用
        (">", true),
        ("$ ", true),
        ("# ", true),
    ];
    
    for (input, expected) in test_cases {
        let result = check_if_should_skip_llm_call(input);
        assert_eq!(result, expected, 
                   "跳过LLM调用测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试新格式进展检测
#[test]
fn test_new_format_progress_detection() {
    let test_cases = vec![
        // 新格式：应该检测到进展
        ("* Processing… (56s · esc to interrupt)", true),
        ("✶ Thinking… (123s · esc to interrupt)", true),
        
        // 标准格式：应该检测到进展
        ("* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)", true),
        
        // 只有时间计数器：不应该检测到进展
        ("56s", false),
        ("123s", false),
        
        // 明显的进展
        ("Tool use: Reading file", true),
        ("✅ 完成", true),
    ];
    
    for (input, expected) in test_cases {
        let result = has_substantial_progress(input);
        assert_eq!(result, expected, 
                   "进展检测测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试Done状态的特殊处理
#[test]
fn test_done_state_handling() {
    let test_cases = vec![
        // Done状态：不应该被认为是活动状态
        ("* Done… (56s · esc to interrupt)", false),
        ("✶ Done… (123s · tokens · esc to interrupt)", false),
        ("Done processing", false),
        
        // 包含Done但不是Done状态：应该是活动状态
        ("* Processing… (56s · esc to interrupt)\nDone something else", true),
        ("* Thinking… (123s · esc to interrupt)\nNot done yet", true),
    ];
    
    for (input, expected) in test_cases {
        let result = is_claude_active(input);
        assert_eq!(result, expected, 
                   "Done状态处理测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试边界情况和错误处理
#[test]
fn test_edge_cases_and_error_handling() {
    let test_cases = vec![
        // 空输入和无效输入
        ("", false),
        ("   ", false),
        ("No format here", false),
        
        // 部分匹配
        ("(esc to interrupt)", false), // 没有执行条前缀
        ("* Processing…", false), // 没有括号内容
        ("* Processing… ()", false), // 空括号
        
        // 格式错误 - 注意：当前实现对这些情况的处理可能比较宽松
        ("* Processing… (invalid s · esc to interrupt)", true), // 仍然匹配执行条格式
        ("* Processing… (56s · esc to interrupt", false), // 缺少右括号
        ("* Processing… 56s · esc to interrupt)", false), // 缺少左括号
        
        // Unicode字符处理
        ("✶ Processing… (56s · esc to interrupt)", true),
        ("✻ Processing… (56s · esc to interrupt)", true),
        ("✽ Processing… (56s · esc to interrupt)", true),
        ("· Processing… (56s · esc to interrupt)", true),
    ];
    
    for (input, expected) in test_cases {
        let result = is_claude_active(input);
        assert_eq!(result, expected, 
                   "边界情况测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试性能：确保新格式不会显著影响性能
#[test]
fn test_new_format_performance() {
    use std::time::Instant;
    
    let test_cases = vec![
        "* Processing… (56s · esc to interrupt)",
        "* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)",
        "✶ Thinking… (123s · esc to interrupt)",
        "No format here",
    ];
    
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        for test_case in &test_cases {
            let _ = is_claude_active(test_case);
            let _ = extract_execution_time(test_case);
            let _ = check_if_should_skip_llm_call(test_case);
            let _ = has_substantial_progress(test_case);
        }
    }
    
    let duration = start.elapsed();
    println!("新格式性能测试: {} 次迭代耗时 {:?}", iterations, duration);
    
    // 确保性能在合理范围内
    assert!(duration.as_millis() < 100, "新格式性能测试失败: 耗时过长");
}

/// 测试多行文本中的新格式
#[test]
fn test_new_format_multiline() {
    let test_cases = vec![
        // 新格式在多行文本中
        (
            "Some previous text\n* Processing… (56s · esc to interrupt)\nMore text",
            true
        ),
        (
            "Line 1\nLine 2\n✶ Thinking… (123s · esc to interrupt)\nLine 4",
            true
        ),
        
        // 混合格式
        (
            "Old format: * Herding… (343s · ↑ 14.2k tokens · esc to interrupt)\nNew format: * Processing… (56s · esc to interrupt)",
            true
        ),
        
        // 没有活动状态
        (
            "Just some text\nMore text\nFinal line\nNo activity here",
            false
        ),
    ];
    
    for (input, expected) in test_cases {
        let result = is_claude_active(input);
        assert_eq!(result, expected, 
                   "多行文本测试失败: input='{}', expected={}", input, expected);
    }
}

/// 测试正则表达式的安全性
#[test]
fn test_regex_safety() {
    // 测试恶意输入不会导致正则表达式崩溃
    let malicious_inputs = vec![
        format!("* Processing… (56s · esc to interrupt{}", "*".repeat(10000)), // 超长输入
        format!("* Processing… ({}s · esc to interrupt)", "a".repeat(10000)), // 超长内容
        format!("* Processing… ({}56s · esc to interrupt)", "(".repeat(1000)), // 嵌套括号
        format!("* Processing… (56s · esc to interrupt){}", "\n".repeat(1000)), // 大量换行
    ];
    
    for input in malicious_inputs {
        // 确保函数不会崩溃
        let _ = is_claude_active(&input);
        let _ = extract_execution_time(&input);
        let _ = check_if_should_skip_llm_call(&input);
        let _ = has_substantial_progress(&input);
    }
}