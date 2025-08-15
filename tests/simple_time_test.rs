//! 简单的时间提取测试

use claude_watch::monitor::extract_execution_time;

#[test]
fn test_simple_time_extraction() {
    // 测试真实数据中的时间格式
    let test_cases = vec![
        ("✻ Philosophising… (475s · ↓ 7 tokens · esc to interrupt)", Some(475)),
        ("* Philosophising… (500s · ↓ 7 tokens · esc to interrupt)", Some(500)),
        ("✽ Philosophising… (352s · ↓ 7 tokens · esc to interrupt)", Some(352)),
        ("* Herding… (169s · ↑ 8.7k tokens · esc to interrupt)", Some(169)),
        ("(123s)", Some(123)),
        ("No time here", None),
    ];
    
    for (input, expected) in test_cases {
        println!("测试: '{}'", input);
        let result = extract_execution_time(input);
        println!("期望: {:?}, 实际: {:?}", expected, result);
        assert_eq!(result, expected, "时间提取失败: '{}'", input);
    }
}