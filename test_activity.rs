use claude_watch::activity::is_claude_active;

fn main() {
    let test_cases = vec![
        "Cogitating...",
        "* Cogitating… (376s · ↓ 4.9k tokens · esc to interrupt)",
        "  /compact",
        "  Retry",
        "  Retry",
        "  Retry",
        "  Retry",
        "  Retry还遇到了这种情况",
    ];

    for (i, test_case) in test_cases.iter().enumerate() {
        let is_active = is_claude_active(test_case);
        println!("测试用例 {}: \"{}\" -> {}", i + 1, test_case, is_active);
    }
}