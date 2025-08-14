//! 详细的Unicode测试

use claude_watch::monitor::extract_execution_time;

fn main() {
    // 测试从文件中复制的确切内容
    let exact_text = "✻ Philosophising… (475s · ↓ 7 tokens · esc to interrupt)";
    
    println!("测试文本: '{}'", exact_text);
    
    // 测试不同的正则表达式
    println!("\n=== 测试不同的正则表达式 ===");
    let patterns = vec![
        r"\((\d+)s[^)]*\)",  // 修复后的模式
        r"\((\d+)s\)",
        r"\((\d+)s",
        r"(\d+)s\)",
        r"\d+s",
        r"475s",
    ];
    
    for pattern in patterns {
        let test_pattern = regex::Regex::new(pattern).unwrap();
        let match_result = test_pattern.captures(exact_text);
        println!("模式 '{}': {:?}", pattern, match_result);
    }
    
    // 测试截断的字符串
    println!("\n=== 测试截断的字符串 ===");
    let truncated = &exact_text[..25]; // 包含 (475s 的部分
    println!("截断字符串: '{}'", truncated);
    let truncated_result = extract_execution_time(truncated);
    println!("截断结果: {:?}", truncated_result);
    
    // 测试手动构造的字符串
    println!("\n=== 测试手动构造的字符串 ===");
    let manual_text = format!("✻ Philosophising… (475s)");
    println!("手动构造: '{}'", manual_text);
    let manual_result = extract_execution_time(&manual_text);
    println!("手动结果: {:?}", manual_result);
    
    // 测试完全ASCII的字符串
    println!("\n=== 测试完全ASCII的字符串 ===");
    let ascii_text = "* Philosophising (475s)";
    println!("ASCII文本: '{}'", ascii_text);
    let ascii_result = extract_execution_time(ascii_text);
    println!("ASCII结果: {:?}", ascii_result);
    
    // 测试修复后的函数
    println!("\n=== 测试修复后的函数 ===");
    let fixed_result = extract_execution_time(exact_text);
    println!("修复后的函数结果: {:?}", fixed_result);
}