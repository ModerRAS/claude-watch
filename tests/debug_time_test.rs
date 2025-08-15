//! 调试时间提取功能

use claude_watch::monitor::extract_execution_time;

#[test]
fn test_debug_time_extraction() {
    let test_text = "✻ Philosophising… (475s · ↓ 7 tokens · esc to interrupt)";
    
    println!("测试文本: '{}'", test_text);
    
    // 手动测试正则表达式
    let time_pattern = regex::Regex::new(r"\((\d+)s\)").unwrap();
    if let Some(caps) = time_pattern.captures(test_text) {
        if let Some(time_str) = caps.get(1) {
            println!("找到时间字符串: '{}'", time_str.as_str());
            if let Ok(time) = time_str.as_str().parse::<u64>() {
                println!("解析成功: {}", time);
            } else {
                println!("解析失败");
            }
        } else {
            println!("没有找到时间组");
        }
    } else {
        println!("没有匹配到时间模式");
    }
    
    // 测试函数
    let result = extract_execution_time(test_text);
    println!("函数结果: {:?}", result);
    
    // 应该能找到时间
    assert!(result.is_some(), "应该能够提取时间");
}